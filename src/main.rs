#![deny(missing_docs)]
//! <fullname>CloudWatch logs processor</fullname>
//!
//! Lambda function that receives log events
//! from CloudWatch Logs. It tries to find who the invocation
//! belongs to, and sends the event to the owner's account.
use aws_sdk_cloudwatchlogs::Client as CwClient;
use aws_sdk_sts::Client as StsClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

mod cloudwatch_logs;
use cloudwatch_logs::*;

mod dynamodb_ext;

mod error;
use error::RuntimeError;

mod event;
use event::*;

mod function_info;

mod dynamodb;
use dynamodb::DynamoDBClient;

mod sts;

#[tracing::instrument(skip(sts_client, dynamodb_client, event))]
async fn handle_logs(
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Get AWS Configuration
    let config = aws_config::load_from_env().await;
    let sts_client = StsClient::new(&config);

    let dynamodb_table =
        std::env::var("DYNAMODB_TABLE").expect("missing environment variable DYNAMODB_TABLE");
    let dynamodb_assume_role = std::env::var("DYNAMODB_ASSUME_ROLE")
        .expect("missing environment variable DYNAMODB_ASSUME_ROLE");

    let session_id = format!("cloudwatch_logs_processor_session_{}", uuid::Uuid::new_v4());
    let dynamodb_config = sts::assume_role(&sts_client, &session_id, &dynamodb_assume_role).await?;
    let dynamodb_client = DynamoDBClient::new(&dynamodb_config, &dynamodb_table).await;

    run(service_fn(|event: LambdaEvent<LogsEvent>| {
        handle_logs(&sts_client, &dynamodb_client, event)
    }))
    .await
}
