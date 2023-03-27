use self::buffer::Buffer;
use self::search_state::{SearchSlice, SearchSourceBuffer, SearchState};
use crate::file_reader::log_entry::{LogEntry, Source};
use crossbeam_channel::Receiver;
use std::{collections::HashSet, ops::Range};

mod buffer;
mod search_state;

#[derive(Debug)]
pub struct PaginationState {
    pub current: usize,
    pub total: Option<usize>,
}

#[derive(Debug)]
pub enum SearchPaginationState {
    NoMatchesFound,
    MatchesIteration(PaginationState),
}

pub struct DataSource {
    pub offset: usize,
    pub selected_index: usize,
    source: EntrySource,
    last_count: usize,
    all_sources: HashSet<Source>,
    seach_state: Option<SearchState>,
}

impl DataSource {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            offset: 0,
            selected_index: 0,
            last_count: 0,
            all_sources: HashSet::new(),
            source: EntrySource::Plain(PlainSource::new(Buffer::new(receiver))),
            seach_state: None,
        }
    }

    pub fn iterate_entries_to_draw<F>(&self, f: F)
    where
        F: Fn((usize, &LogEntry)),
    {
        match &self.source {
            EntrySource::Plain(source) => source.iterate_entries_to_draw(f),
            EntrySource::Filtered(source) => source.iterate_entries_to_draw(f),
        }
    }

    pub fn load_logs(&mut self, height: usize) {
        let buffer_len = match &self.source {
            EntrySource::Plain(source) => source.buffer_len(),
            EntrySource::Filtered(source) => source.buffer_len(),
        };
        let mut request_count = (self.offset + height * 2).saturating_sub(buffer_len);
        while request_count > 0 {
            let entry = match &mut self.source {
                EntrySource::Plain(source) => source.buffer.take_next(),
                EntrySource::Filtered(source) => source.take_next(),
            };
            request_count = match entry {
                Some(entry) => {
                    if !self.all_sources.contains(&entry.source) {
                        self.all_sources.insert(entry.source.clone());
                    }
                    request_count - 1
                }
                None => 0,
            }
        }
    }

    pub fn prepare_for_draw(&mut self, count: usize) {
        self.last_count = count;
        if self.selected_index < self.offset {
            self.offset = self.selected_index;
        } else if self.selected_index >= self.offset + count {
            self.offset += self.selected_index - self.offset - count + 1;
        }
        match self.source {
            EntrySource::Plain(ref mut source) => source.prepare_logs_to_draw(self.offset, count),
            EntrySource::Filtered(ref mut source) => {
                source.prepare_logs_to_draw(self.offset, count)
            }
        }
    }

    pub fn select_next(&mut self) {
        let buffer_len = match &self.source {
            EntrySource::Plain(source) => source.buffer_len(),
            EntrySource::Filtered(source) => source.buffer_len(),
        };
        self.selected_index = self.selected_index.saturating_add(1).min(buffer_len - 1);
        log::info!("Next log selected at index: {}", self.selected_index);
    }

    pub fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
        log::info!("Previous log selected at index: {}", self.selected_index);
    }

    pub fn go_to_next_page(&mut self) {
        let count = self.last_count;
        let start = self.offset + self.last_count;
        self.offset = match &mut self.source {
            EntrySource::Plain(source) => {
                source.prepare_logs_to_draw(start, count);
                source.range.start
            }
            EntrySource::Filtered(source) => {
                source.prepare_logs_to_draw(start, count);
                source.range.start
            }
        };
        self.selected_index = self.offset;
        log::info!("Switched to next page. Offset: {}", self.offset);
    }

    pub fn go_to_prev_page(&mut self) {
        if self.offset >= self.last_count {
            self.offset -= self.last_count;
        } else {
            self.offset = 0;
        }
        self.selected_index = self.offset;
        log::info!("Switched to previous page. Offset: {}", self.offset);
    }

    pub fn set_selected_sources(&mut self, sources: HashSet<u64>) {
        log::info!("Set new selected sources");
        self.offset = 0;
        self.selected_index = 0;
        self.seach_state = None;

        let is_all_sources = sources.is_empty() || sources.len() == self.all_sources.len();
        match &mut self.source {
            EntrySource::Plain(source) if !is_all_sources => {
                let buffer = std::mem::take(&mut source.buffer);
                self.source = EntrySource::Filtered(FilteredSource::new(buffer, sources))
            }
            EntrySource::Filtered(source) if is_all_sources => {
                let buffer = std::mem::take(&mut source.buffer);
                self.source = EntrySource::Plain(PlainSource::new(buffer))
            }
            EntrySource::Filtered(source) if source.selected_sources != sources => {
                let buffer = std::mem::take(&mut source.buffer);
                self.source = EntrySource::Filtered(FilteredSource::new(buffer, sources))
            }
            _ => {}
        }
        self.prepare_for_draw(self.last_count);
    }

    pub fn start_search(&mut self, query: String) {
        log::info!("Search started for query: {query}");
        let mut search_state = SearchState::new(query);
        self.selected_index = match &mut self.source {
            EntrySource::Plain(source) => {
                search_state.start(self.selected_index, &mut source.buffer)
            }
            EntrySource::Filtered(source) => search_state.start(self.selected_index, source),
        };
        log::info!("First selected index: {}", self.selected_index);
        self.seach_state = Some(search_state);
    }

    pub fn stop_search(&mut self) {
        log::info!("Search stopped");
        self.seach_state = None;
    }

    pub fn go_to_next_search_result(&mut self) {
        let search_state = self.seach_state.as_mut().unwrap();
        let index = match &mut self.source {
            EntrySource::Plain(source) => search_state.go_to_next_search_result(&mut source.buffer),
            EntrySource::Filtered(source) => search_state.go_to_next_search_result(source),
        };
        if let Some(index) = index {
            self.selected_index = index;
        }
    }

    pub fn go_to_prev_search_result(&mut self) {
        let search_state = self.seach_state.as_mut().unwrap();
        if let Some(index) = search_state.go_to_prev_search_result() {
            self.selected_index = index;
        }
    }

    pub fn active_message(&self) -> Option<&LogEntry> {
        match &self.source {
            EntrySource::Plain(source) => source.entry(self.selected_index),
            EntrySource::Filtered(source) => source.entry(self.selected_index),
        }
    }

    pub fn search_pagination_state(&self) -> SearchPaginationState {
        self.seach_state
            .as_ref()
            .and_then(|state| {
                state.current_match_index.map(|index| PaginationState {
                    current: index + 1,
                    total: state.is_end_reached.then_some(state.matches_len()),
                })
            })
            .map(SearchPaginationState::MatchesIteration)
            .unwrap_or(SearchPaginationState::NoMatchesFound)
    }

    pub fn pagination_state(&self) -> PaginationState {
        PaginationState {
            current: self.selected_index + 1,
            total: Some(match &self.source {
                EntrySource::Plain(source) => source.buffer_len(),
                EntrySource::Filtered(source) => source.buffer_len(),
            }),
        }
    }

    pub fn iterate_sources<F>(&self, f: F)
    where
        F: FnMut((&Source, bool)),
    {
        match &self.source {
            EntrySource::Plain(_) => self.all_sources.iter().map(|s| (s, true)).for_each(f),
            EntrySource::Filtered(source) => self
                .all_sources
                .iter()
                .map(|s| (s, source.selected_sources.contains(&s.hash)))
                .for_each(f),
        }
    }
}

enum EntrySource {
    Plain(PlainSource),
    Filtered(FilteredSource),
}

struct PlainSource {
    buffer: Buffer,
    range: Range<usize>,
}

impl PlainSource {
    fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            range: Range { start: 0, end: 0 },
        }
    }

    fn prepare_logs_to_draw(&mut self, start: usize, count: usize) {
        let end = self.buffer.len().min(start + count);
        let start = end.saturating_sub(count);
        self.range = Range { start, end };
    }

    fn iterate_entries_to_draw<F>(&self, f: F)
    where
        F: Fn((usize, &LogEntry)),
    {
        self.buffer.inner()[self.range.clone()]
            .iter()
            .enumerate()
            .for_each(f)
    }

    fn entry(&self, index: usize) -> Option<&LogEntry> {
        self.buffer.inner().get(index)
    }

    fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

struct FilteredSource {
    selected_sources: HashSet<u64>,
    indices: Vec<usize>,
    buffer: Buffer,
    range: Range<usize>,
    last_buffer_index: usize,
    is_end_reached: bool,
}

impl FilteredSource {
    fn new(buffer: Buffer, selected_sources: HashSet<u64>) -> Self {
        Self {
            selected_sources,
            indices: Vec::new(),
            buffer,
            range: Range { start: 0, end: 0 },
            last_buffer_index: 0,
            is_end_reached: false,
        }
    }

    fn prepare_logs_to_draw(&mut self, start: usize, count: usize) {
        let start = self.indices.len().min(start);
        if self.is_end_reached || start + count < self.indices.len() {
            let end = self.indices.len().min(start + count);
            let start = end.saturating_sub(count);
            log::info!(
                "Already have indices to draw. Indices len {}, start: {start}, count: {count}",
                self.indices.len()
            );
            self.range = Range { start, end };
        } else {
            let mut found = self.indices.len() - start;
            log::info!(
                "Prepare for draw: indices len: {}, start: {start}, count: {count}",
                self.indices.len()
            );
            while self.take_next().is_some() {
                found += 1;
                if found == count {
                    break;
                }
            }
            log::info!("indices gathered: {:?}", self.indices);
            let len = self.indices.len();
            self.range = Range {
                start: len - start,
                end: len,
            };
        }
    }

    fn iterate_entries_to_draw<F>(&self, f: F)
    where
        F: Fn((usize, &LogEntry)),
    {
        let buffer = self.buffer.inner();
        self.indices[self.range.clone()]
            .iter()
            .map(|index| &buffer[*index])
            .enumerate()
            .for_each(f)
    }

    fn entry(&self, index: usize) -> Option<&LogEntry> {
        let buffer = self.buffer.inner();
        self.indices.get(index).copied().and_then(|i| buffer.get(i))
    }

    fn buffer_len(&self) -> usize {
        self.indices.len()
    }
}

impl SearchSourceBuffer for FilteredSource {
    fn is_end_reached(&self) -> bool {
        self.is_end_reached
    }

    fn len(&self) -> usize {
        self.indices.len()
    }

    fn take_next(&mut self) -> Option<&LogEntry> {
        log::info!("take next. last index {}", self.last_buffer_index);
        let iter = self
            .buffer
            .inner()
            .iter()
            .enumerate()
            .skip(self.last_buffer_index + 1);

        let mut entry_index = None;
        for (index, entry) in iter {
            self.last_buffer_index = index;
            if self.selected_sources.contains(&entry.source.hash) {
                self.indices.push(index);
                entry_index = Some(index);
                break;
            }
        }
        if let Some(index) = entry_index {
            log::info!("Index found: {index}");
            return Some(&self.buffer.inner()[index]);
        }
        if self.buffer.is_end_reached() {
            log::info!("End reached. Search will not continue.");
            self.is_end_reached = true;
            return None;
        }
        loop {
            let len = self.buffer.len();
            match self.buffer.take_next() {
                Some(entry) => {
                    self.last_buffer_index = len;
                    if self.selected_sources.contains(&entry.source.hash) {
                        self.indices.push(self.last_buffer_index);
                        entry_index = Some(self.last_buffer_index);
                        break;
                    }
                }
                None => {
                    self.is_end_reached = true;
                    break;
                }
            }
        }
        log::info!("New entry gathered: {entry_index:?}");
        entry_index.map(|i| &self.buffer.inner()[i])
    }

    fn slice(&self) -> SearchSlice {
        SearchSlice::Filtered(self.buffer.inner(), &self.indices)
    }
}
