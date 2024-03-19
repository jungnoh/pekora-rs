use crate::cache::{CacheKey, CacheLoadResult, CacheableArc};
use chrono::{TimeZone, Utc};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::fs;

pub struct FileBackedCacheableBuilder {
    cache_directory: Arc<PathBuf>,
    cache_max_age: chrono::Duration,
}

impl FileBackedCacheableBuilder {
    pub fn new(cache_directory: Option<String>, cache_max_age: Option<chrono::Duration>) -> Self {
        let cache_directory = cache_directory.unwrap_or("cache".to_string());
        let cache_max_age = cache_max_age.unwrap_or(chrono::Duration::try_days(7).unwrap());

        Self {
            cache_directory: Arc::new(PathBuf::from(Path::new(&cache_directory))),
            cache_max_age,
        }
    }

    pub fn build<I: Send + Sync, O: Serialize + DeserializeOwned + Send + Sync, E: Error>(
        self,
        cacheable: CacheableArc<I, O, E>,
    ) -> FileBackedCacheable<I, O, E> {
        FileBackedCacheable::new(
            cacheable,
            self.cache_max_age,
            self.cache_directory
                .clone()
                .to_str()
                .unwrap_or("")
                .to_string(),
        )
    }
}

pub struct FileBackedCacheable<I: Send + Sync, O: Serialize + DeserializeOwned + Send + Sync, E> {
    cache_directory: Arc<PathBuf>,
    cacheable: CacheableArc<I, O, E>,
    cache_max_age: chrono::Duration,
}

impl<I: Send + Sync, O: Serialize + DeserializeOwned + Send + Sync, E: Error>
    FileBackedCacheable<I, O, E>
{
    pub fn new(
        cacheable: CacheableArc<I, O, E>,
        cache_max_age: chrono::Duration,
        root_path: String,
    ) -> Self {
        Self {
            cacheable,
            cache_max_age,
            cache_directory: Arc::new(PathBuf::from(Path::new(&root_path))),
        }
    }

    pub async fn load(&self, input: &I) -> Result<CacheLoadResult<O>, CacheError<E>> {
        let cache_key = self
            .cacheable
            .get_cache_key(input)
            .await
            .map_err(CacheError::FetchFailed)?;
        debug!("Cache key: {:?}", cache_key);
        if let Some(result) = self.test_cache(&cache_key).await? {
            debug!("Cache hit: {:?}", cache_key);
            return Ok(CacheLoadResult {
                result,
                cache_key: cache_key.clone(),
                cache_hit: true,
            });
        } else {
            debug!("Cache miss: {:?}", cache_key);
        }

        let result = self
            .cacheable
            .load(input)
            .await
            .map_err(CacheError::FetchFailed)?;

        debug!("Writing cache: {:?}", cache_key);
        self.write_cache(&cache_key, &result).await?;
        Ok(CacheLoadResult {
            result,
            cache_key,
            cache_hit: false,
        })
    }

    async fn test_cache(&self, cache_key: &CacheKey) -> Result<Option<O>, CacheError<E>> {
        let usable_file = match self.get_usable_cache_file(cache_key).await? {
            Some(file) => file,
            None => return Ok(None),
        };

        let file = match File::open(usable_file) {
            Ok(file) => file,
            Err(e) => {
                return Err(CacheError::IO(e));
            }
        };

        match file.metadata() {
            Ok(metadata) => {
                let modified = metadata.modified().map_err(CacheError::IO)?;
                let now = chrono::Utc::now();

                let modified_epoch = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
                let modified = Utc.timestamp_opt(modified_epoch as i64, 0).unwrap();
                let age = now.signed_duration_since(modified);
                if age > self.cache_max_age {
                    debug!("Cache expired: {:?}", cache_key);
                    return Ok(None);
                }
            }
            Err(e) => {
                return Err(CacheError::IO(e));
            }
        }

        let reader = BufReader::new(file);
        let result: serde_json::Result<O> = serde_json::from_reader(reader);
        match result {
            Ok(result) => Ok(Some(result)),
            Err(e) => {
                warn!(
                    "Cache deserialization failed, continuing as cache miss: {:?}",
                    e
                );
                Ok(None)
            }
        }
    }

    async fn get_usable_cache_file(
        &self,
        cache_key: &CacheKey,
    ) -> Result<Option<PathBuf>, CacheError<E>> {
        let cache_path = self
            .cache_directory
            .join(self.build_cache_filename(cache_key));
        match File::open(&cache_path) {
            Ok(_) => Ok(Some(cache_path)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                Err(CacheError::IO(e))
            }
        }
    }

    async fn write_cache(&self, cache_key: &CacheKey, result: &O) -> Result<(), CacheError<E>> {
        let cache_path = self
            .cache_directory
            .join(self.build_cache_filename(cache_key));
        if let Some(folder) = cache_path.parent() {
            fs::create_dir_all(folder).await.map_err(CacheError::IO)?;
        }

        // TODO: Use tokio
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(cache_path)
            .map_err(CacheError::IO)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, result).map_err(CacheError::Serde)?;
        Ok(())
    }

    fn build_cache_filename(&self, cache_key: &CacheKey) -> String {
        let filename = match &cache_key.content_key {
            None => match cache_key.content_hash {
                Some(ref hash) => format!("_{}", hash),
                None => {
                    panic!("Cache key must have a content key or hash. This is a bug.");
                }
            },
            Some(content_key) => match cache_key.content_hash {
                Some(ref hash) => format!("{}_{}", content_key, hash),
                None => format!("{}_", content_key),
            },
        };
        format!("{}/{}.json", self.cacheable.category_key(), filename)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError<E: Error> {
    #[error("Cache fetch failed: {0}")]
    FetchFailed(E),
    #[error("Cache IO failed: {0}")]
    Serde(serde_json::Error),
    #[error("Cache IO failed: {0}")]
    IO(std::io::Error),
}

#[cfg(test)]
mod tests {
    use crate::cache::Cacheable;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestObject {
        a: String,
        b: i32,
    }

    struct TestCacheable;

    #[async_trait::async_trait]
    impl Cacheable<String, TestObject, super::CacheError<std::convert::Infallible>> for TestCacheable {
        async fn get_cache_key(
            &self,
            input: &String,
        ) -> Result<Option<String>, super::CacheError<std::convert::Infallible>> {
            Ok(Some(format!("{}-key", input.clone())))
        }

        async fn load(
            &self,
            input: &String,
        ) -> Result<TestObject, super::CacheError<std::convert::Infallible>> {
            Ok(TestObject {
                a: input.clone(),
                b: 42,
            })
        }

        fn category_key(&self) -> String {
            "test".to_string()
        }
    }

    #[tokio::test]
    async fn test_file_backed_cacheable() {
        let cache_key = format!(
            "test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
        );

        let cacheable = super::FileBackedCacheable::new(
            Arc::new(Box::new(TestCacheable)),
            chrono::Duration::try_days(1).unwrap(),
            "test_cache".to_string(),
        );
        let result = cacheable.load(&cache_key.to_string()).await.unwrap();
        assert_eq!(result.result.a, cache_key);
        assert_eq!(result.result.b, 42);
        assert_eq!(
            result.cache_key,
            Some(format!("{}-key", cache_key).to_string())
        );
        assert_eq!(result.cache_hit, false);

        let result = cacheable.load(&cache_key.to_string()).await.unwrap();
        assert_eq!(result.result.a, cache_key);
        assert_eq!(result.result.b, 42);
        assert_eq!(
            result.cache_key,
            Some(format!("{}-key", cache_key).to_string())
        );
        assert_eq!(result.cache_hit, true);
    }
}
