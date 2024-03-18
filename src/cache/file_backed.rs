use async_trait::async_trait;
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

#[async_trait]
pub trait Cacheable<I, O: Serialize + DeserializeOwned + Send + Sync, E: Error> {
    async fn get_cache_key(&self, input: &I) -> Result<Option<String>, E>;
    async fn load(&self, input: &I) -> Result<O, E>;
    fn category_key(&self) -> String;
}

pub struct CacheLoadResult<O> {
    pub result: O,
    pub cache_key: Option<String>,
    pub cache_hit: bool,
}

pub type CacheableArc<I, O, E> = Arc<Box<dyn Cacheable<I, O, E> + Send + Sync>>;

pub struct FileBackedCacheable<I: Send + Sync, O: Serialize + DeserializeOwned + Send + Sync, E> {
    cache_directory: Arc<PathBuf>,
    cacheable: CacheableArc<I, O, E>,
}

impl<I: Send + Sync, O: Serialize + DeserializeOwned + Send + Sync, E: Error>
    FileBackedCacheable<I, O, E>
{
    pub fn new(cacheable: CacheableArc<I, O, E>, root_path: String) -> Self {
        let cache_directory = Path::new(&root_path).join(cacheable.category_key());
        Self {
            cacheable,
            cache_directory: Arc::new(cache_directory),
        }
    }

    pub async fn load(&self, input: &I) -> Result<CacheLoadResult<O>, CacheError<E>> {
        let cache_key = self
            .cacheable
            .get_cache_key(input)
            .await
            .map_err(CacheError::FetchFailed)?;
        debug!("Cache key: {:?}", cache_key);
        if let Some(cache_key) = &cache_key {
            if let Some(result) = self.test_cache(cache_key.clone()).await? {
                debug!("Cache hit: {:?}", cache_key);
                return Ok(CacheLoadResult {
                    result,
                    cache_key: Some(cache_key.clone()),
                    cache_hit: true,
                });
            } else {
                debug!("Cache miss: {:?}", cache_key);
            }
        }

        let result = self
            .cacheable
            .load(input)
            .await
            .map_err(CacheError::FetchFailed)?;
        if let Some(cache_key) = &cache_key {
            debug!("Writing cache: {:?}", cache_key);
            self.write_cache(cache_key, &result).await?;
        }
        Ok(CacheLoadResult {
            result,
            cache_key,
            cache_hit: false,
        })
    }

    async fn test_cache(&self, cache_key: String) -> Result<Option<O>, CacheError<E>> {
        let cache_path = self.cache_directory.join(cache_key);

        // TODO: Use tokio
        let file = match File::open(cache_path) {
            Ok(file) => file,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(CacheError::IO(e));
            }
        };

        let reader = BufReader::new(file);
        let result: serde_json::Result<O> = serde_json::from_reader(reader);
        match result {
            Ok(result) => Ok(Some(result)),
            Err(e) => {
                warn!("Cache deserialization failed, continuing as cache miss: {:?}", e);
                Ok(None)
            }
        }
    }

    async fn write_cache(&self, cache_key: &str, result: &O) -> Result<(), CacheError<E>> {
        fs::create_dir_all(&self.cache_directory.clone().as_path())
            .await
            .map_err(CacheError::IO)?;

        let cache_path = self.cache_directory.join(cache_key);

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
    use crate::cache::file_backed::Cacheable;
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
    impl Cacheable<String, TestObject, super::CacheError<std::convert::Infallible>>
        for TestCacheable
    {
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
        let cache_key = format!("test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros());

        let cacheable = super::FileBackedCacheable::new(
            Arc::new(Box::new(TestCacheable)),
            "test_cache".to_string(),
        );
        let result = cacheable.load(&cache_key.to_string()).await.unwrap();
        assert_eq!(result.result.a, cache_key);
        assert_eq!(result.result.b, 42);
        assert_eq!(result.cache_key, Some(format!("{}-key", cache_key).to_string()));
        assert_eq!(result.cache_hit, false);

        let result = cacheable.load(&cache_key.to_string()).await.unwrap();
        assert_eq!(result.result.a, cache_key);
        assert_eq!(result.result.b, 42);
        assert_eq!(result.cache_key, Some(format!("{}-key", cache_key).to_string()));
        assert_eq!(result.cache_hit, true);
    }
}
