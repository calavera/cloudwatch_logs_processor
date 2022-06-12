use thiserror::Error as ThisError;

/// Different errors that the application can raise
#[derive(Debug, ThisError)]
pub enum RuntimeError {
    /// Error returned when we cannot find the function info in DynamoDB
    #[error("unable to find function information for log group {0}")]
    MissingFunction(String),
    /// Error returned if we cannot assume a specific role
    #[error("failed to assume role")]
    AssumeRoleFailure(#[from] aws_sdk_sts::Error),
    /// Error returned if the credentials are missing after assuming a new role
    #[error("missing cloudwatch credentials")]
    MissingCredentials,
    /// Error returned by the CloudWatch Logs API
    #[error("unexpected cloudwatch logs error")]
    CloudWatchLogs(#[from] aws_sdk_cloudwatchlogs::Error),
    /// Error returned if the function info item in DynamoDB is missing an expected field
    #[error("missing item field {0}")]
    MissingField(String),
    /// Error retuned by the DynamoDB API
    #[error("unexpected dynamodb error")]
    DynamoDB(#[from] aws_sdk_dynamodb::Error),
}
