mod api;
mod cache;
mod util;
mod transform;

use crate::api::aws::ec2::Ec2Client;
use crate::api::aws::elasticache::ElasticacheClient;
use crate::api::aws::price_bulk::{
    PricingListClient, RegionIndexClient, SavingsPlanListClient, ServiceIndexClient,
};
use crate::api::aws::price_bulk_types::{PriceBulkOffer, PriceBulkSavingsPlan};
use crate::cache::FileBackedCacheableBuilder;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum TestCommands {
    ServiceList,
    RegionIndex {
        #[arg(long, default_value = "AmazonEC2")]
        service: String,
    },
    PricingList {
        #[arg(long, default_value = "AmazonEC2")]
        service: String,
        #[arg(long, default_value = "ap-northeast-1")]
        region: String,
        #[arg(long, default_value = "20240312153724")]
        version: String,
    },
    SavingsPlanList {
        #[arg(long, default_value = "AWSComputeSavingsPlan")]
        service: String,
        #[arg(long, default_value = "20240312234047")]
        version: String,
        #[arg(long, default_value = "ap-northeast-1")]
        region: String,
    },
    Ec2AllInstanceTypes,
    RedisTypeSpecificParameters,
    MemcachedTypeSpecificParameters,
}

async fn main_test_command(cmd: &TestCommands) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let cacheable_builder = FileBackedCacheableBuilder::new(None, None);

    match cmd {
        TestCommands::ServiceList {} => {
            let cached =
                cacheable_builder.build(ServiceIndexClient::new_cacheable_arc(client, None));
            println!("{:?}", cached.load(&()).await.unwrap());
        }
        TestCommands::RegionIndex { service } => {
            let cached =
                cacheable_builder.build(RegionIndexClient::new_cacheable_arc(client, None));
            println!("{:?}", cached.load(&service.to_string()).await.unwrap());
        }
        TestCommands::PricingList {
            service,
            region,
            version,
        } => {
            let cached =
                cacheable_builder.build(PricingListClient::new_cacheable_arc(client, None));
            let response = cached
                .load(&PriceBulkOffer {
                    region: region.clone(),
                    service_code: service.clone(),
                    offer_version: version.clone(),
                    filename: "index.json".to_string(),
                })
                .await;
            println!("{:?}", response);
        }
        TestCommands::SavingsPlanList {
            service,
            version,
            region,
        } => {
            let cached =
                cacheable_builder.build(SavingsPlanListClient::new_cacheable_arc(client, None));
            let response = cached
                .load(&PriceBulkSavingsPlan {
                    region: region.clone(),
                    service_code: service.clone(),
                    offer_version: version.clone(),
                    filename: "index.json".to_string(),
                })
                .await?;
            let response = transform::aws::savings_plan::pivot(response.result);
            for item in response {
                println!("{:?}", item);
            }
        }
        TestCommands::Ec2AllInstanceTypes => {
            let ec2_client = Ec2Client::new(None).await;
            let response = ec2_client.describe_all_instance_types().await;
            println!("{:?}", response);
        }
        TestCommands::RedisTypeSpecificParameters => {
            let client = ElasticacheClient::new(None).await;
            let response = client.list_redis_type_specific_parameters().await;
            println!("{:?}", response);
        }
        TestCommands::MemcachedTypeSpecificParameters => {
            let client = ElasticacheClient::new(None).await;
            let response = client.list_memcached_type_specific_parameters().await;
            println!("{:?}", response);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { command } => {
            println!("{:?}", main_test_command(&command).await);
        }
    }
}
