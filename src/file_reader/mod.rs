use crossbeam_channel::Sender;
use log_entry::LogEntry;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod log_entry;

pub fn read_file(file: File, sender: Sender<LogEntry>, callback: cursive::CbSink) {
    let reader = BufReader::new(file);
    let iterator = reader
        .lines()
        .map(|result| result.map(LogEntry::from).unwrap_or(LogEntry::Empty));
    for entry in iterator {
        if sender.is_full() {
            callback.send(Box::new(cursive::Cursive::noop)).unwrap();
        }
        sender.send(entry).unwrap();
    }
    callback.send(Box::new(cursive::Cursive::noop)).unwrap();
}
