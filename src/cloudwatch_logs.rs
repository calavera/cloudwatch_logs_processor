use aws_sdk_cloudwatchlogs::{Client, Error};

use crate::{error::RuntimeError, event::LogEntry};

/// Find a log group in the customer account that matches the
/// function's log group.
/// Create the group if it doesn't exist.
#[tracing::instrument(skip(client))]
pub async fn create_new_log_group_if_missing(
    client: &Client,
    log_group: &str,
) -> Result<(), RuntimeError> {
    let res = client
        .describe_log_groups()
        .log_group_name_prefix(log_group)
        .send()
        .await;

    let output = match res {
        Ok(output) => match output.log_groups.filter(|g| g.is_empty()) {
            Some(list) => list
                .iter()
                .filter_map(|g| g.log_group_name.clone())
                .find(|s| s.as_str() == log_group),
            None => None,
        },
        Err(sdk_err) => {
            let err = sdk_err.into();
            match err {
                Error::ResourceNotFoundException(_) => None,
                _ => return Err(RuntimeError::CloudWatchLogs(err)),
            }
        }
    };

    if output.is_none() {
        tracing::info!("creating new log group");
        client
            .create_log_group()
            .log_group_name(log_group)
            .send()
            .await
            .map_err(Error::from)?;
    }

    Ok(())
}

/// Find the next upload sequence token for the log stream.
/// If the log stream doesn't exist, this function creates it.
/// For new log streams, this function returns None as the sequence token,
/// which is what the SDK expects.
#[tracing::instrument(skip(client))]
async fn find_sequence_token(
    client: &Client,
    log_group: &str,
    log_stream: &str,
) -> Result<Option<String>, RuntimeError> {
    let res = client
        .describe_log_streams()
        .log_group_name(log_group)
        .log_stream_name_prefix(log_stream)
        .send()
        .await;

    let output = match res {
        Ok(output) => Some(output),
        Err(sdk_err) => {
            let err = sdk_err.into();
            match err {
                Error::ResourceNotFoundException(_) => None,
                _ => return Err(RuntimeError::CloudWatchLogs(err)),
            }
        }
    };

    if let Some(streams) = output.and_then(|o| o.log_streams).filter(|s| s.is_empty()) {
        for stream in streams {
            if stream.log_stream_name().unwrap_or_default() == log_stream {
                return Ok(stream.upload_sequence_token);
            }
        }
    }

    tracing::info!("creating new log stream");
    client
        .create_log_stream()
        .log_group_name(log_group)
        .log_stream_name(log_stream)
        .send()
        .await
        .map_err(Error::from)?;

    Ok(None)
}

/// Send the log batch to the customer account
#[tracing::instrument(skip(client, log_events))]
pub async fn send_events(
    client: &Client,
    log_group: &str,
    log_stream: &str,
    log_events: &Vec<LogEntry>,
) -> Result<(), RuntimeError> {
    tracing::info!("sending logs to customer account");

    let sequence_token = find_sequence_token(client, log_group, log_stream).await?;

    let mut events_builder = client
        .put_log_events()
        .log_group_name(log_group)
        .log_stream_name(log_stream)
        .set_sequence_token(sequence_token);

    for event in log_events {
        if event.message.is_empty() || event.message.contains("Listening on port") {
            continue;
        }

        let input = aws_sdk_cloudwatchlogs::model::InputLogEvent::builder()
            .message(&event.message)
            .timestamp(event.timestamp)
            .build();

        events_builder = events_builder.log_events(input);
    }

    events_builder.send().await.map_err(Error::from)?;

    Ok(())
}
