use crossbeam_channel::Sender;
use itertools::Itertools;
use log_entry::LogEntry;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod log_entry;

pub fn read_file(file: File, is_raw_file: bool, sender: Sender<LogEntry>, callback: cursive::CbSink) {
    let reader = BufReader::new(file);
    let parser = if is_raw_file {
        LogEntry::from_raw
    }  else {
        LogEntry::from_json
    };
    let mut iterator = reader
        .lines()
        .filter_map(|result| {
            match result {
                Err(error) => {
                    log::error!("Read log file failed: {error:?}");
                    None
                }
                Ok(line) => {
                    Some(parser(&line).ok_or(line))
                }
            }
        })
        .peekable();

    while let Some(entry) = iterator.next() {
        let Ok(mut entry) = entry else {
            continue;
        };
        while let Some(Err(line)) = iterator.peek() {
            entry.append(line);   
            log::info!("append line {line}");
            iterator.next();
        }
        if sender.is_full() {
            callback.send(Box::new(cursive::Cursive::noop)).unwrap();
        }
        sender.send(entry).unwrap();
    }
    callback.send(Box::new(cursive::Cursive::noop)).unwrap();
}
