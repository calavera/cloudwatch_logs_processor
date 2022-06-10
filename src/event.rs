use serde::{de::Error, Deserialize, Deserializer};
use std::io::BufReader;

/// `LogsEvent` represents the raw event sent by CloudWatch
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LogsEvent {
    // aws_logs is gzipped and base64 encoded, it needs a custom deserializer
    #[serde(rename = "awslogs")]
    pub aws_logs: AwsLogs,
}

/// `AwsLogs` is an unmarshaled, ungzipped, CloudWatch logs event
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AwsLogs {
    #[serde(deserialize_with = "from_base64")]
    pub data: LogData,
}

/// `LogData` represents the logs group event information
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_example() {
        let json = r#"{
    "awslogs": {
        "data": "H4sIAFETomIAA12Ry27bMBBF9/4KQuiyqsQ36Z2DqEGBGC0sdRUHAS0NExV6uCJVNw3y76Fkx03CFTH3cubwztMChRO14Jy5h+JxD9ESRZerYnW3zvJ8dZVFn4+W/tDBMImYUMaFVDrF5FVs+vuroR/3k56Yg0sa0+4qk0D50MddX8Ev98aa+wFMO3lJinWS0gTT5ObT9arI8uJWM2uUkMCpZIxiorGRtsQMiOXCgHxt5MadK4d67+u++1o3HgYXWt7M4my4nhmOw+7Kph+rg/HlQwBwM1M0W2//c2V/oPPvmzydb7OpriZqygQhFItUa6GlUkymgrNUS5EKpQhRfMpGCEzC/xgWjCpNOBMn8nM3X4fcvWmn2DDnhGNFWXiffvCdtjON3mQ/vm8KtIHfY3j6rVoiEdaxsxZizLSJd4KRWGFrYwIKqBSVMtZu/eU4mCmoJWLii2KodVt/UTcNVOiNJEMdbf0a2n54RHn9DwKYJmh9EYrmLzoJPx2EwfJY33bRmfb5mOjiefECiB5LsVgCAAA="
    }
}"#;
        let event: LogsEvent = serde_json::from_str(json).expect("failed to deserialize");
        let data = event.aws_logs.data;
        assert_eq!("DATA_MESSAGE", data.message_type);
        assert_eq!("123456789012", data.owner);
        assert_eq!("/aws/lambda/echo-nodejs", data.log_group);
        assert_eq!(
            "2019/03/13/[$LATEST]94fa867e5374431291a7fc14e2f56ae7",
            data.log_stream
        );
        assert_eq!(1, data.subscription_filters.len());
        assert_eq!(
            "LambdaStream_cloudwatchlogs-node",
            data.subscription_filters[0]
        );
        assert_eq!(1, data.log_events.len());
        assert_eq!(
            "34622316099697884706540976068822859012661220141643892546",
            data.log_events[0].id
        );
        assert_eq!(1552518348220, data.log_events[0].timestamp);
        assert_eq!("REPORT RequestId: 6234bffe-149a-b642-81ff-2e8e376d8aff\tDuration: 46.84 ms\tBilled Duration: 47 ms \tMemory Size: 192 MB\tMax Memory Used: 72 MB\t\n", data.log_events[0].message);
    }
}
