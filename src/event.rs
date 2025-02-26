use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, io::BufReader};

/// `LogsEvent` represents the raw event sent by CloudWatch
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LogsEvent {
    /// `aws_logs` is gzipped and base64 encoded, it needs a custom deserializer
    #[serde(rename = "awslogs")]
    pub aws_logs: AwsLogs,
}

/// `AwsLogs` is an unmarshaled, ungzipped, CloudWatch logs event
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AwsLogs {
    /// `data` is the log data after is decompressed
    pub data: LogData,
}

/// `LogData` represents the logs group event information
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogData {
    /// Owner of the log event
    pub owner: String,
    /// Log group where the event was published
    pub log_group: String,
    /// Log stream where the event was published
    pub log_stream: String,
    /// Filters applied to the event
    pub subscription_filters: Vec<String>,
    /// Type of event
    pub message_type: String,
    /// Entries in the log batch
    pub log_events: Vec<LogEntry>,
}

/// `LogEntry` represents a log entry from cloudwatch logs
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LogEntry {
    /// Unique id for the entry
    pub id: String,
    /// Time when the event was published
    pub timestamp: i64,
    /// Message published in the application log
    pub message: String,
}

impl<'de> Deserialize<'de> for AwsLogs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AwsLogsVisitor;

        impl<'de> Visitor<'de> for AwsLogsVisitor {
            type Value = AwsLogs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base64 gzipped string")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AwsLogs, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "data" => {
                            let bytes = map.next_value::<String>().and_then(|string| {
                                base64::decode(&string).map_err(Error::custom)
                            })?;

                            let bytes = flate2::read::GzDecoder::new(&bytes[..]);
                            let mut de =
                                serde_json::Deserializer::from_reader(BufReader::new(bytes));
                            data = Some(LogData::deserialize(&mut de).map_err(Error::custom)?);
                        }
                        _ => return Err(Error::unknown_field(key, FIELDS)),
                    }
                }

                let data = data.ok_or_else(|| Error::missing_field("data"))?;
                Ok(AwsLogs { data })
            }
        }

        const FIELDS: &[&str] = &["data"];
        deserializer.deserialize_struct("AwsLogs", FIELDS, AwsLogsVisitor)
    }
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
