use std::collections::HashMap;
use std::sync::Arc;
use anyhow::bail;
use chrono::{DateTime, Utc};
use log::warn;
use crate::api::aws::price_bulk_types::SavingsPlanListResponse;
use crate::api::aws::types::{LeaseContractLength, SavingsPlanProductAttributes, SavingsPlanTermRate};

#[derive(Debug, Clone)]
pub struct PivotedSavingsPlanTermRate {
    pub savings_plan_sku: String,
    pub savings_plan_effective_date: DateTime<Utc>,
    pub savings_plan_attributes: Arc<SavingsPlanProductAttributes>,
    pub lease_contract_length: LeaseContractLength,
    pub term_rate: SavingsPlanTermRate,
}

pub fn pivot(response: SavingsPlanListResponse) -> anyhow::Result<Vec<PivotedSavingsPlanTermRate>> {
    let mut attribute_lookup: HashMap<String, Arc<SavingsPlanProductAttributes>> = HashMap::new();
    for product in response.products {
        attribute_lookup.insert(product.sku, Arc::new(product.attributes));
    }

    let mut pivoted: Vec<PivotedSavingsPlanTermRate> = Vec::new();
    for term in response.terms.savings_plan {
        for rate in term.rates {
            let attributes = match attribute_lookup.get(&term.sku) {
                Some(attributes) => attributes.clone(),
                None => {
                    bail!("No attributes found for savings plan sku {}", term.sku);
                }
            };
            pivoted.push(PivotedSavingsPlanTermRate {
                savings_plan_sku: term.sku.clone(),
                savings_plan_effective_date: term.effective_date,
                savings_plan_attributes: attributes.clone(),
                lease_contract_length: term.lease_contract_length.clone(),
                term_rate: rate,
            });
        }
    }
    Ok(pivoted)
}