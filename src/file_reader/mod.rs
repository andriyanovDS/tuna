use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use log_entry::LogEntry;

pub mod log_entry;

pub fn read_file(file: File, sender: Sender<LogEntry>, callback: cursive::CbSink) {
    let reader = BufReader::new(file);
    let iterator = reader
        .lines()
        .map(|result| result.map(LogEntry::from).unwrap_or(LogEntry::Empty));
    for entry in iterator {
        sender.send(entry).unwrap();
        callback.send(Box::new(cursive::Cursive::noop)).unwrap();
    }
}
