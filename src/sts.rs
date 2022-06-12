use crate::error::RuntimeError;
use aws_sdk_iam::Credentials;
use aws_sdk_sts::{Client, Error};
use aws_types::SdkConfig;

/// Assume a new role to perform operations in a different account.
///
/// TODO(david): is the assume_role_arn considered private information that
/// we cannot have in our service logs? if it's private, add it to the `skip` attribute
/// in the instrument macro below.
#[tracing::instrument(skip(client))]
pub async fn assume_role(
    client: &Client,
    session_id: &str,
    assume_role_arn: &str,
) -> Result<SdkConfig, RuntimeError> {
    tracing::info!("assuming new role role");

    let assumed_role = client
        .assume_role()
        .role_arn(assume_role_arn)
        .role_session_name(session_id)
        .send()
        .await
        .map_err(Error::from)?;

    let credentials = match assumed_role.credentials {
        Some(creds) => creds,
        None => return Err(RuntimeError::MissingCredentials),
    };

    let (access_key_id, secret_access_key) =
        match (credentials.access_key_id(), credentials.secret_access_key()) {
            (Some(id), Some(key)) => (id, key),
            _ => return Err(RuntimeError::MissingCredentials),
        };

    let assumed_credentials = Credentials::from_keys(
        access_key_id,
        secret_access_key,
        credentials.session_token.clone(),
    );

    let new_config = aws_config::from_env()
        .credentials_provider(assumed_credentials)
        .load()
        .await;

    Ok(new_config)
}
