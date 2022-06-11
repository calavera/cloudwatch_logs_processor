#![deny(missing_docs)]
//! <fullname>CloudWatch logs forwarder</fullname>
//! Lambda function that receives log events
//! from CloudWatch. Tries to find who the invocation
//! belongs to, and sends the event to the owner's account.
use aws_sdk_sts::Client as StsClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

mod cloudwatch_logs;
use cloudwatch_logs::*;

mod dynamodb_ext;

mod error;
use error::RuntimeError;

mod event;
use event::*;

mod store;

#[tracing::instrument(skip(client, store, event))]
async fn handle_logs(
    client: &StsClient,
    store: &store::DynamoDBStore,
    event: LambdaEvent<LogsEvent>,
) -> Result<(), RuntimeError> {
    let session_id = event.context.request_id;
    let data = event.payload.aws_logs.data;
    let log_group = data.log_group;

    let function_id = match log_group.rsplit('/').next() {
        Some(id) => id,
        _ => return Err(RuntimeError::MissingFunction(log_group)),
    };

    // TODO(david): Find function information in the dynamodbstore
    let info = store::FunctionInfo {
        id: function_id.into(),
        name: "app-id-brach-name".into(),
        logs_assume_role_arn: "assume_role_arn".into(),
    };

    // Initialize CloudWatch logs client with assumed credentials
    let cw_customer_client =
        assume_cw_customer_role(client, &session_id, &info.logs_assume_role_arn).await?;

    // replace aws/lambda/... with our own log prefix
    let new_log_group = format!("aws/amplify/compute/{}", &info.name);
    create_new_log_group_if_missing(&cw_customer_client, &new_log_group).await?;

    send_events(
        &cw_customer_client,
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
    let store = store::get_store(&config).await;

    run(service_fn(|event: LambdaEvent<LogsEvent>| {
        handle_logs(&sts_client, &store, event)
    }))
    .await
}
