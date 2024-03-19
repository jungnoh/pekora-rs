use crate::api::aws::types::{
    PriceOffering, RITermAttributes, SavingPlanProduct, SavingsPlanTerms,
};
use crate::util::regex_extract_match_group;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceListResponse {
    pub format_version: String,
    pub publication_date: DateTime<Utc>,
    pub offers: HashMap<String, ServiceListResponseOffer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceListResponseOffer {
    pub offer_code: String,
    pub version_index_url: Option<String>,
    pub current_version_url: Option<String>,
    pub current_region_index_url: Option<String>,
    pub savings_plan_version_index_url: Option<String>,
    pub current_savings_plan_index_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionIndexResponse {
    pub format_version: String,
    pub publication_date: DateTime<Utc>,
    pub regions: HashMap<String, RegionIndexResponseRegion>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionIndexResponseRegion {
    pub region_code: String,
    pub current_version_url: PriceBulkOffer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse<PT: Debug + Clone, TT: Debug + Clone> {
    pub format_version: String,
    pub publication_date: DateTime<Utc>,
    pub version: String,
    pub products: PT,
    pub terms: TT,
}

pub type PricingListResponse = ProductResponse<
    HashMap<String, PricingListResponseProduct<HashMap<String, String>>>,
    PricingListResponseTerms,
>;

pub type SavingsPlanListResponse = ProductResponse<Vec<SavingPlanProduct>, SavingsPlanTerms>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingListResponseProduct<T: Debug + Clone> {
    pub product_family: String,
    pub sku: String,
    pub attributes: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PricingListResponseTerms {
    #[serde(rename = "OnDemand")]
    pub on_demand: HashMap<String, HashMap<String, PriceOffering<HashMap<String, String>>>>,
    #[serde(rename = "Reserved")]
    pub reserved: HashMap<String, HashMap<String, PriceOffering<RITermAttributes>>>,
}

// ======= Utility types - not part of the response DTO ==========
lazy_static! {
    static ref OFFER_RESOURCE_REGEX: Regex =
        Regex::new(r"^\/([^/]+)\/v1.0\/aws\/([^/]+)\/([^/]+)\/([^/]+)\/([^/]+)$").unwrap();
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceBulkOffer {
    pub service_code: String,
    pub offer_version: String,
    pub region: String,
    pub filename: String,
}

impl PriceBulkOffer {
    pub fn path(&self) -> String {
        format!(
            "offers/v1.0/aws/{}/{}/{}/{}",
            self.service_code, self.offer_version, self.region, self.filename
        )
    }

    pub fn tag(&self) -> String {
        format!(
            "{}-{}-{}",
            self.region, self.service_code, self.offer_version,
        )
    }
}

impl<'de> Deserialize<'de> for PriceBulkOffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        PriceBulkOffer::try_from(s.to_string()).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for PriceBulkOffer {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let captures = match OFFER_RESOURCE_REGEX.captures(&s) {
            Some(captures) => captures,
            None => anyhow::bail!("Invalid offer resource path: {}", s),
        };

        let service_code = regex_extract_match_group(&captures, 2, "service_code")?;
        let offer_version = regex_extract_match_group(&captures, 3, "offer_version")?;
        let region = regex_extract_match_group(&captures, 4, "region")?;
        let filename = regex_extract_match_group(&captures, 5, "filename")?;

        Ok(PriceBulkOffer {
            service_code,
            offer_version,
            region,
            filename,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PriceBulkSavingsPlan {
    pub service_code: String,
    pub offer_version: String,
    pub region: String,
    pub filename: String,
}

impl PriceBulkSavingsPlan {
    pub fn tag(&self) -> String {
        format!(
            "{}-{}-{}",
            self.region, self.service_code, self.offer_version,
        )
    }

    pub fn path(&self) -> String {
        format!(
            "savingsPlan/v1.0/aws/{}/{}/{}/{}",
            self.service_code, self.offer_version, self.region, self.filename
        )
    }
}

impl<'de> Deserialize<'de> for PriceBulkSavingsPlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        PriceBulkSavingsPlan::try_from(s.to_string()).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for PriceBulkSavingsPlan {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let captures = match OFFER_RESOURCE_REGEX.captures(&s) {
            Some(captures) => captures,
            None => anyhow::bail!("Invalid offer resource path: {}", s),
        };

        let service_code = regex_extract_match_group(&captures, 2, "service_code")?;
        let offer_version = regex_extract_match_group(&captures, 3, "offer_version")?;
        let region = regex_extract_match_group(&captures, 4, "region")?;
        let filename = regex_extract_match_group(&captures, 5, "filename")?;

        Ok(PriceBulkSavingsPlan {
            service_code,
            offer_version,
            region,
            filename,
        })
    }
}
