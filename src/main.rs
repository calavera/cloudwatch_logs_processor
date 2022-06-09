#![warn(missing_docs)]
//! <fullname>CloudWatch logs forwarder</fullname>
//! Lambda function that receives log events
//! from CloudWatch. Tries to find who the invocation
//! belongs to, and sends the event to the owner's account.
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

mod dynamodb_ext;
mod event;
use event::*;
mod store;

async fn handle_logs(store: &store::DynamoDBStore, event: LambdaEvent<LogsEvent>) -> Result<(), Error> {
    let payload = event.payload;
    let function_id = payload.aws_logs.log_group.rsplitn(2, "/").next();

    // - Find function information in the dynamodbstore
    // - Assume customer role
    //        See https://github.com/awslabs/aws-sdk-rust/blob/bd5fa57de41af37d56b56f1dc72604637da0e504/examples/iam/src/bin/iam-getting-started.rs#L191
    // - Initialize CloudWatch logs client with assumed credentials
    // - Send payload to to CloudWatch
    //      - Filter start/stop server logs
    //      - Find or create log group
    //      - Find or create log stream
    //      - Put log events
    //           See https://docs.rs/aws-sdk-cloudwatchlogs/latest/aws_sdk_cloudwatchlogs/client/struct.Client.html#method.put_log_events

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let store = store::get_store().await;
    run(service_fn(|event: LambdaEvent<LogsEvent>| handle_logs(&store, event))).await
}
