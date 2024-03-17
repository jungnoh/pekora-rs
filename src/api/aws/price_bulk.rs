use crate::api::aws::price_bulk_types::*;
use log::debug;

pub struct PriceBulkClient<'a> {
    client: reqwest::Client,
    base_url: &'a str,
}

impl<'a> PriceBulkClient<'a> {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client: client.clone(),
            base_url: "https://pricing.us-east-1.amazonaws.com",
        }
    }

    pub async fn get_service_list(&self) -> PriceBulkResult<ServiceListResponse> {
        let response = self.build_request("offers/v1.0/aws/index.json").await?;
        Ok(response.json::<ServiceListResponse>().await?)
    }

    pub async fn get_region_index(
        &self,
        service_code: &str,
    ) -> PriceBulkResult<RegionIndexResponse> {
        let path = format!("offers/v1.0/aws/{}/current/region_index.json", service_code);
        let response = self.build_request(&path).await?;
        Ok(response.json::<RegionIndexResponse>().await?)
    }

    pub async fn get_pricing_list(
        &self,
        resource: &PriceBulkOffer,
    ) -> PriceBulkResult<PricingListResponse> {
        let path = resource.path();
        let response = self.build_request(&path).await?;
        Ok(response.json::<PricingListResponse>().await?)
    }

    pub async fn get_savings_plan_list(
        &self,
        resource: &PriceBulkSavingsPlan,
    ) -> PriceBulkResult<SavingsPlanListResponse> {
        let path = resource.path();
        let response = self.build_request(&path).await?;
        Ok(response.json::<SavingsPlanListResponse>().await?)
    }

    async fn build_request(&self, path: &str) -> PriceBulkResult<reqwest::Response> {
        let url = format!("{}/{}", self.base_url, path);
        debug!("Requesting URL: {}", url);
        let request = self.client.get(&url).build().map_err(PriceBulkError::from);
        let response = self
            .client
            .execute(request?)
            .await
            .map_err(PriceBulkError::from)?;
        response
            .error_for_status()
            .map_err(PriceBulkError::HttpResponseFailure)
    }
}

pub type PriceBulkResult<T> = Result<T, PriceBulkError>;

#[derive(thiserror::Error, Debug)]
pub enum PriceBulkError {
    #[error("HTTP client failure: {0}")]
    HttpFailure(#[from] reqwest::Error),
    #[error("HTTP response error: {0}")]
    HttpResponseFailure(reqwest::Error),
}
