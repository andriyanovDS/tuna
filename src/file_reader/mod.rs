use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use anyhow::Error;

pub enum LogEntry {
    Empty,
}

impl From<String> for LogEntry {
    fn from(value: String) -> Self {
        Self::Empty
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item=LogEntry>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let iterator = reader
        .lines()
        .map(|result| {
            result
                .map(LogEntry::from)
                .unwrap_or(LogEntry::Empty)
        });
    Ok(iterator)
}
