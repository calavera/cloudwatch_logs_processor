use serde::{de::Error, Deserialize, Deserializer};
use std::io::BufReader;

/// `LogsEvent` represents the raw event sent by CloudWatch
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LogsEvent {
    // aws_logs is gzipped and base64 encoded, it needs a custom deserializer
    #[serde(rename = "awslogs", deserialize_with = "from_base64")]
    pub aws_logs: LogData,
}

/// `LogData` is an unmarshaled, ungzipped, CloudWatch logs event
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogData {
    pub owner: String,
    pub log_group: String,
    pub log_stream: String,
    pub subscription_filters: Vec<String>,
    pub message_type: String,
    pub log_events: Vec<LogEntry>,
}

/// `LogEntry` represents a log entry from cloudwatch logs
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub message: String,
}

fn from_base64<'d, D>(deserializer: D) -> Result<LogData, D::Error>
where
    D: Deserializer<'d>,
{
    let bytes = String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(D::Error::custom))?;

    let bytes = flate2::read::GzDecoder::new(&bytes[..]);
    let mut de = serde_json::Deserializer::from_reader(BufReader::new(bytes));
    LogData::deserialize(&mut de).map_err(D::Error::custom)
}
