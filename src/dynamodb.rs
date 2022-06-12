use crate::{dynamodb_ext::*, error::RuntimeError, function_info::FunctionInfo};
use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use std::collections::HashMap;

/// DynamoDB client implementation.
pub struct DynamoDBClient {
    inner: Client,
    table: String,
}

impl DynamoDBClient {
    /// Initialize the DynamoDB store.
    #[tracing::instrument(skip(config))]
    pub async fn new(config: &aws_types::SdkConfig, table: &str) -> DynamoDBClient {
        tracing::info!("Initializing DynamoDB client");
        let inner = aws_sdk_dynamodb::Client::new(config);
        DynamoDBClient {
            inner,
            table: table.into(),
        }
    }

    /// Fetch the function information from DynamoDB to locate the assume role arn.
    #[tracing::instrument(skip(self))]
    pub async fn get_function_info(&self, id: &str) -> Result<FunctionInfo, RuntimeError> {
        let res = self
            .inner
            .get_item()
            .table_name(&self.table)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await
            .map_err(Error::from)?;

        res.item
            .ok_or_else(|| RuntimeError::MissingFunction(id.into()))
            .and_then(|i| i.try_into())
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for FunctionInfo {
    type Error = RuntimeError;

    /// Try to convert a DynamoDB item into a FunctionInfo.
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(FunctionInfo {
            id: value
                .get_s("id")
                .ok_or_else(|| RuntimeError::MissingField("id".into()))?,
            name: value
                .get_s("name")
                .ok_or_else(|| RuntimeError::MissingField("name".into()))?,
            cloudwatch_logs_assume_role_arn: value
                .get_s("cloudwatch_logs_assume_role_arn")
                .ok_or_else(|| {
                    RuntimeError::MissingField("cloudwatch_logs_assume_role_arn".into())
                })?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;
    use aws_sdk_dynamodb::{Client, Config};
    use aws_smithy_client::{erase::DynConnector, test_connection::TestConnection};
    use aws_smithy_http::body::SdkBody;

    #[tokio::test]
    async fn test_get_function_info() -> Result<(), RuntimeError> {
        // GIVEN a DynamoDBClient with one item
        let conn = TestConnection::new(vec![(
            get_request_builder("dynamodb")
                .header("content-type", "application/x-amz-json-1.0")
                .header("x-amz-target", "DynamoDB_20120810.GetItem")
                .body(SdkBody::from(r#"{"TableName": "test", "Key": {"id": {"S": "1"}}}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Item": {"id": {"S": "1"}, "name": {"S": "app-id-1-branch-2"}, "cloudwatch_logs_assume_role_arn": {"S": "arn"}}}"#))
                .unwrap(),
        )]);
        let config = Config::new(&get_mock_config().await);
        let inner = Client::from_conf_conn(config, DynConnector::new(conn.clone()));

        let store = DynamoDBClient {
            inner,
            table: "test".to_string(),
        };

        // WHEN getting an item
        let function = store.get_function_info("1").await?;

        // THEN the response has the correct values
        assert_eq!("1", function.id);
        assert_eq!("app-id-1-branch-2", function.name);
        assert_eq!("arn", function.cloudwatch_logs_assume_role_arn);

        // AND the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }
}
