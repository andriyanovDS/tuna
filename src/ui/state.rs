use crate::file_reader::log_entry::LogEntry;
use crossbeam_channel::Receiver;
use cursive::theme::{BaseColor, ColorStyle, PaletteColor, PaletteStyle, StyleType};

pub struct Styles {
    pub time_style: StyleType,
    pub source_style: StyleType,
    pub msg_style: StyleType,
    pub msg_style_hl: StyleType,
}

impl Styles {
    pub fn new() -> Self {
        Self {
            time_style: ColorStyle::new(BaseColor::Yellow, PaletteColor::Background).into(),
            source_style: ColorStyle::new(BaseColor::Blue, PaletteColor::Background).into(),
            msg_style: ColorStyle::new(PaletteColor::Primary, PaletteColor::Background).into(),
            msg_style_hl: PaletteStyle::Highlight.into(),
        }
    }
}

#[derive(Debug)]
pub enum MatchesSearchState {
    NoMatchesFound,
    MatchesIteration(PaginationState),
}

#[derive(Debug)]
pub struct PaginationState {
    pub current: usize,
    pub total: Option<usize>,
}

pub struct LogsPanelState {
    pub offset: usize,
    pub selected_index: usize,
    pub styles: Styles,
    buffer: Vec<LogEntry>,
    receiver: Receiver<LogEntry>,
    search_query: Option<String>,
    ascending_match_indices: Vec<usize>,
    descending_match_indices: Vec<usize>,
    current_match: usize,
    last_height: usize,
}

impl LogsPanelState {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            selected_index: 0,
            receiver,
            styles: Styles::new(),
            search_query: None,
            ascending_match_indices: Vec::new(),
            descending_match_indices: Vec::new(),
            last_height: 0,
            current_match: 0,
        }
    }

    pub fn logs_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn log_iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.buffer.iter()
    }

    pub fn load_logs(&mut self, screen_height: usize) {
        let diff = (self.offset + screen_height * 2).saturating_sub(self.buffer.len());
        let mut request_count = diff;
        while request_count > 0 {
            if let Ok(entry) = self.receiver.recv() {
                self.buffer.push(entry);
                request_count -= 1;
            } else {
                request_count = 0;
            }
        }
    }

    pub fn adjust_offset(&mut self, screen_height: usize) {
        self.last_height = screen_height.saturating_sub(2);
        let selected_index = self.selected_index;
        let max_y = self.last_height;
        let offset = self.offset;
        if selected_index < offset {
            self.offset = selected_index;
        } else if selected_index >= offset + max_y {
            self.offset += selected_index - offset - max_y + 1;
        }
    }

    pub fn go_to_next_page(&mut self) {
        let last_page_offset = self.buffer.len() - self.last_height;
        let next_offset = self.offset + self.last_height;
        self.offset = next_offset.min(last_page_offset);
        self.selected_index = self.offset;
    }

    pub fn go_to_prev_page(&mut self) {
        if self.offset >= self.last_height {
            self.offset -= self.last_height;
        } else {
            self.offset = 0;
        }
        self.selected_index = self.offset;
    }

    pub fn exit_search_mode(&mut self) {
        self.search_query = None;
        self.current_match = 0;
        self.ascending_match_indices.clear();
        self.descending_match_indices.clear();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = Some(query.to_lowercase());
        self.current_match = 0;
        self.ascending_match_indices.clear();
        self.descending_match_indices.clear();
        self.go_to_next_search_result();
    }

    pub fn go_to_next_search_result(&mut self) {
        let Some(index) = self.descending_match_indices.last().copied() else {
            self.find_next_log();
            return;
        };
        if index == self.selected_index {
            self.ascending_match_indices.push(index);
            self.descending_match_indices.pop();
        }
        if let Some(index) = self.descending_match_indices.last().copied() {
            self.set_selected_index(index);
            self.current_match += 1;
        } else {
            self.find_next_log();
        }
    }

    pub fn go_to_prev_search_result(&mut self) {
        let Some(index) = self.ascending_match_indices.last().copied() else {
            return;
        };
        if index == self.selected_index {
            self.descending_match_indices.push(index);
            self.ascending_match_indices.pop();
        }
        if let Some(index) = self.ascending_match_indices.last().copied() {
            self.set_selected_index(index);
            self.current_match -= 1;
        }
    }

    pub fn matches_search_state(&self) -> Option<MatchesSearchState> {
        self.search_query.as_ref().map(|_| {
            if self.current_match == 0 {
                MatchesSearchState::NoMatchesFound
            } else {
                let state = PaginationState {
                    current: self.current_match,
                    total: self.receiver.is_empty().then_some({
                        self.ascending_match_indices.len() + self.descending_match_indices.len()
                    }),
                };
                MatchesSearchState::MatchesIteration(state)
            }
        })
    }

    pub fn pagination_state(&self) -> PaginationState {
        PaginationState {
            current: self.selected_index + 1,
            total: Some(self.buffer.len()),
        }
    }

    pub fn active_message(&self) -> &LogEntry {
        &self.buffer[self.selected_index]
    }

    fn find_next_log(&mut self) {
        let Some(query) = self.search_query.as_ref() else {
            return;
        };
        let start_index = self
            .ascending_match_indices
            .last()
            .map(|index| (index + 1).min(self.buffer.len() - 1))
            .unwrap_or(0);

        let index = self
            .buffer
            .iter()
            .skip(start_index)
            .enumerate()
            .find_map(|(index, entry)| entry.contains(query).then_some(index))
            .map(|i| i + start_index)
            .or_else(|| {
                while let Ok(entry) = self.receiver.recv() {
                    let contains = entry.contains(query);
                    self.buffer.push(entry);
                    if contains {
                        return Some(self.buffer.len() - 1);
                    }
                }
                None
            });
        if let Some(index) = index {
            self.set_selected_index(index);
            self.current_match += 1;
            self.ascending_match_indices.push(index);
        }
    }

    fn set_selected_index(&mut self, index: usize) {
        self.selected_index = index;
        if self.offset + self.last_height < self.selected_index {
            self.offset = index;
        }
    }
}

impl LogEntry {
    fn contains(&self, query: &String) -> bool {
        self.lower_case_message.contains(query)
    }
}
