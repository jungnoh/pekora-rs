mod api;
mod util;

use crate::api::aws::price_bulk::PriceBulkClient;
use crate::api::aws::price_bulk_types::{PriceBulkOffer, PriceBulkSavingsPlan};
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
}

async fn main_test_command(cmd: &TestCommands) {
    let client = reqwest::Client::new();
    let price_bulk_client = PriceBulkClient::new(client);

    match cmd {
        TestCommands::ServiceList {} => {
            let response = price_bulk_client.get_service_list().await;
            println!("{:?}", response);
        }
        TestCommands::RegionIndex { service } => {
            let response = price_bulk_client.get_region_index(service).await;
            println!("{:?}", response);
        }
        TestCommands::PricingList {
            service,
            region,
            version,
        } => {
            let response = price_bulk_client
                .get_pricing_list(&PriceBulkOffer {
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
            let response = price_bulk_client
                .get_savings_plan_list(&PriceBulkSavingsPlan {
                    service_code: service.clone(),
                    offer_version: version.clone(),
                    region: region.clone(),
                    filename: "index.json".to_string(),
                })
                .await;
            println!("{:?}", response);
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { command } => {
            main_test_command(&command).await;
        }
    }
}
