use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CacheKey {
    pub content_key: Option<String>,
    pub content_hash: Option<String>,
}

#[async_trait]
pub trait Cacheable<I, O: Serialize + DeserializeOwned + Send + Sync, E: Error> {
    async fn get_cache_key(&self, input: &I) -> Result<CacheKey, E>;
    async fn load(&self, input: &I) -> Result<O, E>;
    fn category_key(&self) -> String;
}

#[derive(Debug)]
pub struct CacheLoadResult<O> {
    pub result: O,
    pub cache_key: CacheKey,
    pub cache_hit: bool,
}

pub type CacheableArc<I, O, E> = Arc<Box<dyn Cacheable<I, O, E> + Send + Sync>>;
