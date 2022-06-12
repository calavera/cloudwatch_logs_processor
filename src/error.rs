use thiserror::Error as ThisError;

/// Different errors that the application can raise
#[derive(Debug, ThisError)]
pub enum RuntimeError {
    #[error("unable to find function information for log group {0}")]
    MissingFunction(String),
    #[error("failed to assume role")]
    AssumeRoleFailure(#[from] aws_sdk_sts::Error),
    #[error("missing cloudwatch credentials")]
    MissingCredentials,
    #[error("unexpected cloudwatch logs error")]
    CloudWatchLogs(#[from] aws_sdk_cloudwatchlogs::Error),
    #[error("missing item field {0}")]
    MissingField(String),
    #[error("unexpected dynamodb error")]
    DynamoDB(#[from] aws_sdk_dynamodb::Error),
}
