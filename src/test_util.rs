use aws_sdk_iam::Credentials;
use aws_types::{region::Region, SdkConfig};

/// Configuration for mocking AWS SDK clients
pub async fn get_mock_config() -> SdkConfig {
    aws_config::from_env()
        .region(Region::new("us-west-1"))
        .credentials_provider(Credentials::new(
            "accesskey",
            "privatekey",
            None,
            None,
            "dummy",
        ))
        .load()
        .await
}

/// Base request builder for the AWS SDK calls
pub fn get_request_builder(service: &str) -> http::request::Builder {
    http::Request::builder().uri(format!("https://{service}.us-west-1.amazonaws.com/"))
}
