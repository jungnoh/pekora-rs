use crate::api::aws::util::{AwsClientError, AwsClientResult};
use crate::util::ClientSet;
use aws_config::{BehaviorVersion, SdkConfig};
use aws_sdk_elasticache::types::CacheNodeTypeSpecificParameter;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;

async fn build_client_set(
    aws_sdk_config: Option<SdkConfig>,
) -> ClientSet<SdkConfig, aws_sdk_elasticache::Client> {
    let config = match aws_sdk_config {
        Some(config) => config,
        None => aws_config::load_defaults(BehaviorVersion::latest()).await,
    };
    ClientSet::new(
        config,
        Box::new(|config, region| {
            let mut builder = config.into_builder();
            builder.set_region(aws_config::Region::new(region));
            let new_config = builder.build();
            aws_sdk_elasticache::Client::new(&new_config)
        }),
    )
}

/// Map of (instance type) -> (parameter name) -> (parameter value)
pub type TypeSpecificParameters = HashMap<String, HashMap<String, String>>;

pub struct ElasticacheClient {
    client_set: ClientSet<SdkConfig, aws_sdk_elasticache::Client>,
}

impl ElasticacheClient {
    pub async fn new(aws_sdk_config: Option<SdkConfig>) -> Self {
        Self {
            client_set: build_client_set(aws_sdk_config).await,
        }
    }

    pub async fn list_redis_type_specific_parameters(
        &self,
    ) -> AwsClientResult<TypeSpecificParameters> {
        self.list_cache_node_type_specific_parameters("redis7")
            .await
    }

    pub async fn list_memcached_type_specific_parameters(
        &self,
    ) -> AwsClientResult<TypeSpecificParameters> {
        self.list_cache_node_type_specific_parameters("memcached1.6")
            .await
    }

    pub async fn list_cache_node_type_specific_parameters(
        &self,
        parameter_group_family: &str,
    ) -> AwsClientResult<TypeSpecificParameters> {
        let client = self.client_set.get("us-east-1").await;

        let result =
            list_cache_node_type_specific_parameters(client, parameter_group_family).await?;

        let mut result_map = HashMap::new();
        for parameter in result {
            let parameter_name = match &parameter.parameter_name {
                Some(name) => name,
                None => continue,
            };
            for item in parameter
                .cache_node_type_specific_values
                .unwrap_or(Vec::new())
            {
                let instance_type = match &item.cache_node_type {
                    Some(instance_type) => instance_type,
                    None => continue,
                };
                let parameter_value = match &item.value {
                    Some(parameter_value) => parameter_value,
                    None => continue,
                };
                result_map
                    .entry(instance_type.clone())
                    .or_insert(HashMap::new())
                    .insert(parameter_name.clone(), parameter_value.clone());
            }
        }
        Ok(result_map)
    }
}

async fn list_cache_node_type_specific_parameters(
    client: Arc<aws_sdk_elasticache::Client>,
    parameter_group_family: &str,
) -> AwsClientResult<Vec<CacheNodeTypeSpecificParameter>> {
    info!(
        "ElasticacheClient: DescribeEngineDefaultParameters for {}",
        parameter_group_family
    );
    let request = client
        .describe_engine_default_parameters()
        .set_cache_parameter_group_family(Some(parameter_group_family.to_string()))
        .into_paginator();

    let mut stream = request.send();
    let mut result = Vec::new();

    while let Some(page_result) = stream.next().await {
        match page_result {
            Ok(page) => {
                let engine_defaults = match page.engine_defaults {
                    Some(parameters) => parameters,
                    None => continue,
                };
                result.extend(
                    engine_defaults
                        .cache_node_type_specific_parameters
                        .unwrap_or(Vec::new()),
                );
            }
            Err(e) => return Err(AwsClientError::DescribeEngineDefaultParametersFailure(e)),
        }
    }
    Ok(result)
}
