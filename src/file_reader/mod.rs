use log_entry::LogEntry;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;

pub mod log_entry;

pub fn read_file(file: File, sender: Sender<LogEntry>, callback: cursive::CbSink) {
    let reader = BufReader::new(file);
    let iterator = reader
        .lines()
        .map(|result| result.map(LogEntry::from).unwrap_or(LogEntry::Empty));
    for (index, entry) in iterator.enumerate() {
        sender.send(entry).unwrap();
        if index.wrapping_rem(50) == 0 {
            callback.send(Box::new(cursive::Cursive::noop)).unwrap();
        }
    }
    callback.send(Box::new(cursive::Cursive::noop)).unwrap();
}
