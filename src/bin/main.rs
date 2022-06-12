use aws_sdk_sts::Client as StsClient;
use cloudwatch_log_processor::{handle_logs, sts, DynamoDBClient, LogsEvent};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

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
