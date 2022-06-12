#![deny(missing_docs)]
//! <fullname>CloudWatch logs processor</fullname>
//!
//! Lambda function that receives log events
//! from CloudWatch Logs. It tries to find who the invocation
//! belongs to, and sends the event to the owner's account.
use aws_sdk_cloudwatchlogs::Client as CwClient;
use aws_sdk_sts::Client as StsClient;
use lambda_runtime::LambdaEvent;

mod cloudwatch_logs;
use cloudwatch_logs::*;

mod dynamodb_ext;

mod error;
pub use error::RuntimeError;

mod event;
pub use event::LogsEvent;

mod function_info;

mod dynamodb;
pub use dynamodb::DynamoDBClient;

/// `sts` includes helpers to work with AWS STS
pub mod sts;

#[cfg(test)]
mod test_util;

/// `handle_logs` is the Lambda function entry point
/// that receives the events from CloudWatch Logs
#[tracing::instrument(skip(sts_client, dynamodb_client, event))]
pub async fn handle_logs(
    sts_client: &StsClient,
    dynamodb_client: &DynamoDBClient,
    event: LambdaEvent<LogsEvent>,
) -> Result<(), RuntimeError> {
    let session_id = event.context.request_id;
    let data = event.payload.aws_logs.data;
    let log_group = data.log_group;

    let function_id = match log_group.rsplit('/').next() {
        Some(id) => id,
        _ => return Err(RuntimeError::MissingFunction(log_group)),
    };

    let info = dynamodb_client.get_function_info(function_id).await?;

    // Initialize CloudWatch logs client with assumed credentials
    let cw_config = sts::assume_role(
        sts_client,
        &session_id,
        &info.cloudwatch_logs_assume_role_arn,
    )
    .await?;
    let cw_client = CwClient::new(&cw_config);

    // replace aws/lambda/... with our own log prefix
    let new_log_group = format!("aws/amplify/compute/{}", &info.name);
    create_new_log_group_if_missing(&cw_client, &new_log_group).await?;

    send_events(
        &cw_client,
        &new_log_group,
        &data.log_stream,
        &data.log_events,
    )
    .await
}
