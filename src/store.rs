use crate::dynamodb_ext::*;
use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, instrument};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub logs_assume_role_arn: String,
}

/// DynamoDB store implementation.
pub struct DynamoDBStore {
    client: Client,
    table_name: String,
}

impl DynamoDBStore {
    fn new(client: Client, table_name: String) -> DynamoDBStore {
        DynamoDBStore { client, table_name }
    }

    /// Fetch the function information from DynamoDB to locate the assume role arn.
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> Result<Option<FunctionInfo>, Error> {
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(match res.item {
            Some(item) => Some(item.try_into().unwrap()), // TODO(david): handle this error
            None => None,
        })
    }
}

/// Initialize the DynamoDB store.
#[instrument]
pub async fn get_store(config: &aws_types::SdkConfig) -> DynamoDBStore {
    // Initialize a DynamoDB store
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    info!(
        "Initializing DynamoDB store with table name: {}",
        table_name
    );
    let client = aws_sdk_dynamodb::Client::new(&config);
    DynamoDBStore::new(client, table_name)
}

impl TryFrom<HashMap<String, AttributeValue>> for FunctionInfo {
    type Error = String;

    /// Try to convert a DynamoDB item into a FunctionInfo.
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(FunctionInfo {
            id: value.get_s("id").ok_or("Missing id".to_string())?,
            name: value.get_s("name").ok_or("Missing name".to_string())?,
            logs_assume_role_arn: value
                .get_s("logs_assume_role_arn")
                .ok_or("Missing logs assume role arn".to_string())?,
        })
    }
}
