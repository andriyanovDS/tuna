use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

pub enum LogEntry {
    Empty,
    ParseFailed(LogEntryParseFailed),
    Info(LogMessage),
}

pub struct LogEntryParseFailed {
    pub error_message: String,
}

impl LogEntryParseFailed {
    fn from<E: std::error::Error>(error: E) -> Self {
        Self {
            error_message: error.to_string(),
        }
    }
}

pub struct LogMessage {
    pub message: String,
    pub date: DateTime<FixedOffset>,
    pub date_time: String,
    pub source: String,
    pub list_message: String,
}

impl From<ExternalLogMessage> for LogMessage {
    fn from(value: ExternalLogMessage) -> Self {
        let date_time = value.date.format("%T");
        let date_time = format!("{date_time}");
        let one_line_message = value.message.lines().next().unwrap();
        let list_message = format!("{} [{}] {}", date_time, value.source, one_line_message);
        Self {
            message: value.message,
            date: value.date,
            date_time,
            source: value.source,
            list_message,
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
        serde_json::from_str::<ExternalLogMessage>(&value)
            .map(LogMessage::from)
            .map(Self::Info)
            .unwrap_or_else(|error| {
                log::error!("{error:?}");
                Self::ParseFailed(LogEntryParseFailed::from(error))
            })
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
