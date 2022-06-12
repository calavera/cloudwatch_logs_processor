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
