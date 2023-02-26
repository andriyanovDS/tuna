use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

pub enum LogEntry {
    Empty,
    Info(LogMessage),
}

pub struct LogMessage {
    pub message: String,
    pub date: DateTime<FixedOffset>,
    pub date_time: String,
    pub source: String,
}

impl From<ExternalLogMessage> for LogMessage {
    fn from(value: ExternalLogMessage) -> Self {
        let date_time = value.date.format("%T");
        Self {
            message: value.message,
            date: value.date,
            date_time: format!("{date_time}"),
            source: value.source,
        }
    }
}

#[derive(Deserialize)]
struct ExternalLogMessage {
    message: String,
    #[serde(with = "date_parse")]
    date: DateTime<FixedOffset>,
    source: String,
}

impl From<String> for LogEntry {
    fn from(value: String) -> Self {
        let message = serde_json::from_str::<ExternalLogMessage>(&value)
            .map(LogMessage::from)
            .unwrap();
        Self::Info(message)
    }
}

mod date_parse {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'a, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'a>,
    {
        let string = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc2822(&string).map_err(serde::de::Error::custom)
    }
}
