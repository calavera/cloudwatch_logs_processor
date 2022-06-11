use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum RuntimeError {
    #[error("unable to find function information for log group {0}")]
    MissingFunction(String),
    #[error("failed to assume role")]
    AssumeRoleFailure(#[from] aws_sdk_sts::types::SdkError<aws_sdk_sts::error::AssumeRoleError>),
    #[error("missing cloudwatch credentials")]
    MissingCredentials,
    #[error("unexpected cloudwatch logs error")]
    CloudWatchLogs(#[from] aws_sdk_cloudwatchlogs::Error),
}
