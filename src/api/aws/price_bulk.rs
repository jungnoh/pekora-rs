use crate::api::aws::price_bulk_types::*;
use crate::cache::{Cacheable, CacheableArc, CacheKey};
use async_trait::async_trait;
use log::debug;
use std::sync::Arc;

const DEFAULT_BASE_URL: &str = "https://pricing.us-east-1.amazonaws.com";

pub struct ServiceIndexClient {
    client: reqwest::Client,
    base_url: Arc<String>,
}

#[async_trait]
impl Cacheable<(), ServiceListResponse, PriceBulkError> for ServiceIndexClient {
    async fn get_cache_key(&self, _input: &()) -> Result<CacheKey, PriceBulkError> {
        let request_url = self.request_url();
        let content_hash = load_etag(self.client.clone(), request_url.as_str()).await?;
        Ok(CacheKey {
            content_key: None,
            content_hash,
        })
    }

    async fn load(&self, _input: &()) -> Result<ServiceListResponse, PriceBulkError> {
        let request_url = self.request_url();
        let response = send_request(self.client.clone(), request_url.as_str()).await?;
        Ok(response.json::<ServiceListResponse>().await?)
    }

    fn category_key(&self) -> String {
        "aws/bulk/service_index".to_string()
    }
}

impl ServiceIndexClient {
    pub fn new_cacheable_arc(
        client: reqwest::Client,
        base_url: Option<String>,
    ) -> CacheableArc<(), ServiceListResponse, PriceBulkError> {
        let instance = Self {
            client,
            base_url: Arc::new(base_url.unwrap_or(DEFAULT_BASE_URL.to_string())),
        };
        Arc::new(Box::new(instance))
    }

    fn request_url(&self) -> String {
        format!("{}/offers/v1.0/aws/index.json", self.base_url)
    }
}

pub struct RegionIndexClient {
    client: reqwest::Client,
    base_url: Arc<String>,
}

#[async_trait]
impl Cacheable<String, RegionIndexResponse, PriceBulkError> for RegionIndexClient {
    async fn get_cache_key(&self, service_code: &String) -> Result<CacheKey, PriceBulkError> {
        let request_url = self.request_url(service_code);
        let content_hash = load_etag(self.client.clone(), request_url.as_str()).await?;
        Ok(CacheKey {
            content_key: Some(service_code.clone()),
            content_hash,
        })
    }

    async fn load(&self, service_code: &String) -> Result<RegionIndexResponse, PriceBulkError> {
        let request_url = self.request_url(service_code);
        let response = send_request(self.client.clone(), request_url.as_str()).await?;
        Ok(response.json::<RegionIndexResponse>().await?)
    }

    fn category_key(&self) -> String {
        "aws/bulk/region_index".to_string()
    }
}

impl RegionIndexClient {
    pub fn new_cacheable_arc(
        client: reqwest::Client,
        base_url: Option<String>,
    ) -> CacheableArc<String, RegionIndexResponse, PriceBulkError> {
        let instance = Self {
            client,
            base_url: Arc::new(base_url.unwrap_or(DEFAULT_BASE_URL.to_string())),
        };
        Arc::new(Box::new(instance))
    }

    fn request_url(&self, service_code: &String) -> String {
        format!(
            "{}/offers/v1.0/aws/{}/current/region_index.json",
            self.base_url, service_code
        )
    }
}

pub struct PricingListClient {
    client: reqwest::Client,
    base_url: Arc<String>,
}

#[async_trait]
impl Cacheable<PriceBulkOffer, PricingListResponse, PriceBulkError> for PricingListClient {
    async fn get_cache_key(
        &self,
        input: &PriceBulkOffer,
    ) -> Result<CacheKey, PriceBulkError> {
        let request_url = format!("{}/{}", self.base_url, input.path());
        Ok(CacheKey {
            content_key: Some(input.tag()),
            content_hash: load_etag( self.client.clone(), request_url.as_str()).await?,
        })
    }

    async fn load(&self, input: &PriceBulkOffer) -> Result<PricingListResponse, PriceBulkError> {
        let request_url = format!("{}/{}", self.base_url, input.path());
        let response = send_request(self.client.clone(), request_url.as_str()).await?;
        Ok(response.json::<PricingListResponse>().await?)
    }

    fn category_key(&self) -> String {
        "aws/bulk/pricing_list".to_string()
    }
}

impl PricingListClient {
    pub fn new_cacheable_arc(
        client: reqwest::Client,
        base_url: Option<String>,
    ) -> CacheableArc<PriceBulkOffer, PricingListResponse, PriceBulkError> {
        let instance = Self {
            client,
            base_url: Arc::new(base_url.unwrap_or(DEFAULT_BASE_URL.to_string())),
        };
        Arc::new(Box::new(instance))
    }
}

pub struct SavingsPlanListClient {
    client: reqwest::Client,
    base_url: Arc<String>,
}

#[async_trait]
impl Cacheable<PriceBulkSavingsPlan, SavingsPlanListResponse, PriceBulkError>
    for SavingsPlanListClient
{
    async fn get_cache_key(
        &self,
        input: &PriceBulkSavingsPlan,
    ) -> Result<CacheKey, PriceBulkError> {
        let request_url = format!("{}/{}", self.base_url, input.path());
        Ok(CacheKey {
            content_key: Some(input.tag()),
            content_hash: load_etag( self.client.clone(), request_url.as_str()).await?,
        })
    }

    async fn load(
        &self,
        input: &PriceBulkSavingsPlan,
    ) -> Result<SavingsPlanListResponse, PriceBulkError> {
        let request_url = format!("{}/{}", self.base_url, input.path());
        let response = send_request(self.client.clone(), request_url.as_str()).await?;
        Ok(response.json::<SavingsPlanListResponse>().await?)
    }

    fn category_key(&self) -> String {
        "aws/bulk/savings_plan_list".to_string()
    }
}

impl SavingsPlanListClient {
    pub fn new_cacheable_arc(
        client: reqwest::Client,
        base_url: Option<String>,
    ) -> CacheableArc<PriceBulkSavingsPlan, SavingsPlanListResponse, PriceBulkError> {
        let instance = Self {
            client,
            base_url: Arc::new(base_url.unwrap_or(DEFAULT_BASE_URL.to_string())),
        };
        Arc::new(Box::new(instance))
    }
}

async fn load_etag(client: reqwest::Client, url: &str) -> Result<Option<String>, PriceBulkError> {
    let response = client.head(url).send().await?;
    match response.headers().get("etag").map(|v| v.to_str()) {
        Some(Ok(etag)) => Ok(Some(etag.to_string().trim_matches('"').to_string())),
        _ => Ok(None),
    }
}

async fn send_request(client: reqwest::Client, url: &str) -> PriceBulkResult<reqwest::Response> {
    debug!("Requesting URL: {}", url);
    let request = client.get(url).build().map_err(PriceBulkError::from);
    let response = client
        .execute(request?)
        .await
        .map_err(PriceBulkError::from)?;
    response
        .error_for_status()
        .map_err(PriceBulkError::HttpResponseFailure)
}

pub type PriceBulkResult<T> = Result<T, PriceBulkError>;

#[derive(thiserror::Error, Debug)]
pub enum PriceBulkError {
    #[error("HTTP client failure: {0}")]
    HttpFailure(#[from] reqwest::Error),
    #[error("HTTP response error: {0}")]
    HttpResponseFailure(reqwest::Error),
}
