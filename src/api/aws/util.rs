use aws_sdk_ec2::error::SdkError;
use aws_sdk_ec2::operation::describe_instance_types::DescribeInstanceTypesError;
use aws_sdk_elasticache::operation::describe_engine_default_parameters::DescribeEngineDefaultParametersError;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MAJOR_REGIONS: [&'static str; 5] = [
        "us-west-2",
        "us-east-1",
        "us-east-2",
        "ap-northeast-1",
        "eu-central-1"
    ];
}

pub type AwsClientResult<T> = Result<T, AwsClientError>;

#[derive(thiserror::Error, Debug)]
pub enum AwsClientError {
    #[error("EC2 DescribeInstanceTypes failed: {0}")]
    DescribeInstanceTypesFailure(#[from] SdkError<DescribeInstanceTypesError>),
    #[error("Elasticache DescribeCacheParameters failed: {0}")]
    DescribeEngineDefaultParametersFailure(#[from] SdkError<DescribeEngineDefaultParametersError>),
    #[error("Tokio thread error: {0}")]
    Tokio(#[from] tokio::task::JoinError),
}
