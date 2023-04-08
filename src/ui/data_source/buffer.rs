use super::search_state::{SearchSlice, SearchSourceBuffer};
use crate::file_reader::log_entry::LogEntry;
use crossbeam_channel::Receiver;

#[derive(Default)]
pub struct Buffer {
    buffer: Vec<LogEntry>,
    receiver: Option<Receiver<LogEntry>>,
}

impl Buffer {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            receiver: Some(receiver),
        }
    }

    pub fn inner(&self) -> &[LogEntry] {
        &self.buffer
    }
}

impl SearchSourceBuffer for Buffer {
    fn is_end_reached(&self) -> bool {
        self.receiver.as_ref().unwrap().is_empty()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn take_next(&mut self) -> Option<&LogEntry> {
        let receiver = self.receiver.as_mut().unwrap();
        if let Ok(entry) = receiver.recv() {
            self.buffer.push(entry);
            self.buffer.last()
        } else {
            None
        }
    }

    fn slice(&self) -> SearchSlice {
        SearchSlice::Plain(&self.buffer)
    }
}
