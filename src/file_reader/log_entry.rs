use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct LogEntry {
    pub message: String,
    pub date: DateTime<FixedOffset>,
    pub date_time: String,
    pub source: Source,
    pub one_line_message: String,
    pub lower_case_message: String,
    pub lines_count: usize,
    date_full: Option<String>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub hash: u64,
}

impl Source {
    fn new(name: String) -> Self {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        Self {
            name,
            hash: hasher.finish(),
        }
    }
}

impl From<ExternalLogMessage> for LogEntry {
    fn from(value: ExternalLogMessage) -> Self {
        let date_time = value.date.format("%T%.3f");
        let one_line_message = value.message.lines().next().unwrap().into();
        let lower_case_message = value.message.to_lowercase();
        let lines_count = value.message.lines().count();
        Self {
            message: value.message,
            date: value.date,
            date_time: date_time.to_string(),
            source: Source::new(value.source),
            one_line_message,
            lower_case_message,
            lines_count,
            date_full: None,
        }
    }
}

#[derive(Deserialize, Debug)]
struct ExternalLogMessage {
    message: String,
    #[serde(with = "date_parse")]
    date: DateTime<FixedOffset>,
    source: String,
}

impl LogEntry {
    pub fn from_raw(log: &str) -> Option<Self> {
        let mut iter = log.splitn(3, |c: char| c.is_whitespace());
        let (date, source, message) = (iter.next()?, iter.next()?, iter.next()?);
        if date.is_empty() || source.len() < 3 {
            return None;
        }
        NaiveDateTime::parse_from_str(&date[0..date.len() - 1], "%Y-%m-%dT%H:%M:%S%.3f")
            .map(|date| ExternalLogMessage {
                message: message.to_string(),
                date: DateTime::<FixedOffset>::from_utc(date, FixedOffset::east_opt(0).unwrap()),
                source: source[1..source.len() - 2].to_string(),
            })
            .map(LogEntry::from)
            .ok()
    }

    pub fn from_json(log: &str) -> Option<Self> {
        let result = serde_json::from_str::<ExternalLogMessage>(log).map(LogEntry::from);
        match result {
            Ok(entry) => Some(entry),
            Err(error) => {
                log::error!("Failed to parse log {log:?} with error: {error:?}");
                None
            }
        }
    }

    pub fn append(&mut self, message: &str) {
        self.message.push('\n');
        self.message.push_str(message);
        self.lines_count += 1;
    }

    pub fn date_full(&mut self) -> String {
        if let Some(date) = self.date_full.clone() {
            date
        } else {
            self.date_full = Some(self.date.format("%c").to_string());
            self.date_full.clone().unwrap()
        }
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
        let format = "%e %b %Y %H:%M:%S%.3f %z";  
        DateTime::parse_from_str(string.as_str(), format)
            .map_err(serde::de::Error::custom)
    }
}
