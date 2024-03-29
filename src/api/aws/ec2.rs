use crate::api::aws::util::{AwsClientError, AwsClientResult, MAJOR_REGIONS};
use crate::util::ClientSet;
use aws_config::{BehaviorVersion, SdkConfig};
use aws_sdk_ec2::types::InstanceTypeInfo;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;

async fn build_client_set(
    aws_sdk_config: Option<SdkConfig>,
) -> ClientSet<SdkConfig, aws_sdk_ec2::Client> {
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
            aws_sdk_ec2::Client::new(&new_config)
        }),
    )
}

pub struct Ec2Client {
    client_set: ClientSet<SdkConfig, aws_sdk_ec2::Client>,
}

impl Ec2Client {
    pub async fn new(aws_sdk_config: Option<SdkConfig>) -> Self {
        Self {
            client_set: build_client_set(aws_sdk_config).await,
        }
    }

    pub async fn describe_all_instance_types(
        &self,
    ) -> AwsClientResult<HashMap<String, InstanceTypeInfo>> {
        let mut tasks = Vec::with_capacity(MAJOR_REGIONS.len());
        for region in MAJOR_REGIONS.iter() {
            let client = self.client_set.get(region).await;
            tasks.push(tokio::spawn(describe_instance_types(client, None)));
        }

        let mut result_map = HashMap::new();
        for task_handle in tasks {
            let instance_types = task_handle.await.map_err(AwsClientError::Tokio)??;
            for (k, v) in instance_types {
                result_map.entry(k).or_insert(v);
            }
        }
        Ok(result_map)
    }
}

async fn describe_instance_types(
    client: Arc<aws_sdk_ec2::Client>,
    instance_types: Option<Vec<String>>,
) -> AwsClientResult<HashMap<String, InstanceTypeInfo>> {
    let mut request = client.describe_instance_types();

    if let Some(instance_types) = instance_types {
        let instance_type_enums = instance_types
            .iter()
            .map(|f| aws_sdk_ec2::types::InstanceType::from(f.as_str()))
            .collect::<Vec<_>>();
        request = request.set_instance_types(Some(instance_type_enums));
    }

    let mut result_map = HashMap::new();
    let mut next_token: Option<String> = None;
    loop {
        info!(
            "Ec2Client: Requesting DescribeInstanceTypes (region={:?})",
            client.config().region(),
        );
        let result = request
            .clone()
            .set_next_token(next_token.clone())
            .send()
            .await
            .map_err(AwsClientError::DescribeInstanceTypesFailure)?;
        next_token = result.next_token;
        if result.instance_types.is_none() {
            break;
        }
        let instance_types = result.instance_types.unwrap();
        info!(
            "Ec2Client: Found DescribeInstanceTypes (region={:?}, count={})",
            client.config().region(),
            instance_types.len()
        );
        for ref item in instance_types {
            if let Some(instance_type) = item.instance_type.as_ref() {
                result_map.insert(instance_type.to_string(), item.clone());
            }
        }
        if next_token.is_none() {
            break;
        }
    }
    Ok(result_map)
}
