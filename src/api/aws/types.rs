use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ContractLength {
    #[serde(alias = "1yr", alias = "1 yr")]
    OneYear,
    #[serde(alias = "3yr", alias = "3 yr")]
    ThreeYear,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PurchaseOption {
    #[serde(alias = "No Upfront")]
    NoUpfront,
    #[serde(alias = "Partial Upfront")]
    PartialUpfront,
    #[serde(alias = "All Upfront")]
    AllUpfront,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RIOfferingClass {
    Standard,
    Convertible,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    #[allow(clippy::upper_case_acronyms)]
    USD,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceOffering<TA: Debug + Clone> {
    pub offer_term_code: String,
    pub sku: String,
    pub effective_date: DateTime<Utc>,
    pub price_dimensions: HashMap<String, PriceDimension>,
    pub term_attributes: TA,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceDimension {
    pub rate_code: String,
    pub description: String,
    pub unit: String,
    pub price_per_unit: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RITermAttributes {
    #[serde(rename = "LeaseContractLength")]
    pub lease_contract_length: ContractLength,
    #[serde(rename = "OfferingClass")]
    pub offering_class: RIOfferingClass,
    #[serde(rename = "PurchaseOption")]
    pub purchase_option: PurchaseOption,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingPlanProduct {
    pub sku: String,
    pub product_family: String,
    pub service_code: String,
    pub usage_type: String,
    pub operation: String,
    pub attributes: SavingsPlanProductAttributes,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingsPlanProductAttributes {
    pub purchase_option: PurchaseOption,
    pub product_family: String,
    pub region_code: Option<String>,
    pub service_code: String,
    pub granularity: String,
    pub instance_type: Option<String>,
    pub location_type: String,
    pub purchase_term: ContractLength,
    pub location: String,
    pub usage_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingsPlanTerms {
    pub savings_plan: Vec<SavingsPlanTerm>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingsPlanTerm {
    pub sku: String,
    pub description: String,
    pub effective_date: DateTime<Utc>,
    pub lease_contract_length: LeaseContractLength,
    pub rates: Vec<SavingsPlanTermRate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LeaseContractLength {
    pub duration: i32,
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingsPlanTermRate {
    pub discounted_sku: String,
    pub discounted_usage_type: String,
    pub discounted_operation: String,
    pub discounted_service_code: String,
    pub rate_code: String,
    pub unit: String,
    pub discounted_rate: DiscountedRate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscountedRate {
    pub price: String,
    pub currency: Currency,
}
