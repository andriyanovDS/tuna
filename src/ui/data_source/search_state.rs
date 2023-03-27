use crate::file_reader::log_entry::LogEntry;

pub struct SearchState {
    query: String,
    match_indices: Vec<usize>,
    pub current_match_index: Option<usize>,
    pub is_end_reached: bool,
}

pub enum SearchSlice<'v> {
    Plain(&'v [LogEntry]),
    Filtered(&'v [LogEntry], &'v [usize]),
}

pub trait SearchSourceBuffer {
    fn is_end_reached(&self) -> bool;
    fn len(&self) -> usize;
    fn take_next(&mut self) -> Option<&LogEntry>;
    fn slice(&self) -> SearchSlice;
}

impl SearchState {
    pub fn new(query: String) -> Self {
        Self {
            query: query.to_lowercase(),
            match_indices: Vec::new(),
            current_match_index: None,
            is_end_reached: false,
        }
    }

    pub fn matches_len(&self) -> usize {
        self.match_indices.len()
    }

    pub fn start<B: SearchSourceBuffer>(&mut self, selected_index: usize, buffer: &mut B) -> usize {
        let current_index = selected_index;
        let mut selected_index = selected_index;
        loop {
            self.go_to_next_search_result(buffer);
            let Some(index) = self.current_match_index else {
                break;
            };
            let match_index = self.match_indices[index];
            if match_index < current_index && !buffer.is_end_reached() {
                continue;
            } else if match_index > current_index {
                let prev_index = self.match_indices[index.saturating_sub(1)];
                let closest_index =
                    if prev_index.abs_diff(current_index) < (match_index - current_index) {
                        prev_index
                    } else {
                        match_index
                    };
                selected_index = closest_index;
            } else {
                selected_index = match_index;
            }
            break;
        }
        selected_index
    }

    pub fn go_to_next_search_result<B: SearchSourceBuffer>(
        &mut self,
        buffer: &mut B,
    ) -> Option<usize> {
        let index = self
            .current_match_index
            .map(|i| i + 1)
            .filter(|i| *i < self.match_indices.len());
        if let Some(index) = index {
            self.current_match_index = Some(index);
            Some(self.match_indices[index])
        } else {
            self.find_next(buffer)
        }
    }

    pub fn go_to_prev_search_result(&mut self) -> Option<usize> {
        if let Some(index) = self.current_match_index.map(|v| v.saturating_sub(1)) {
            self.current_match_index = Some(index);
            Some(self.match_indices[index])
        } else {
            None
        }
    }

    fn find_next<B: SearchSourceBuffer>(&mut self, buffer: &mut B) -> Option<usize> {
        if self.is_end_reached {
            log::info!("All search results were found");
            return None;
        }
        let start_index = self
            .current_match_index
            .map(|index| self.match_indices[index] + 1)
            .unwrap_or(0);

        let query = self.query.as_str();
        let index = match buffer.slice() {
            SearchSlice::Filtered(slice, indices) => {
                log::info!(
                    "Filtered source. Slice len: {}, indices len: {}",
                    slice.len(),
                    indices.len()
                );
                let iter = indices.iter().copied().map(|i| &slice[i]);
                SearchState::find_next_index(query, iter, start_index)
            }
            SearchSlice::Plain(slice) => {
                SearchState::find_next_index(query, slice.iter(), start_index)
            }
        };
        log::info!("Found next search index in cached data: {index:?}");
        let index = index.or_else(|| {
            while let Some(entry) = buffer.take_next() {
                if entry.lower_case_message.contains(query) {
                    let index = buffer.len() - 1;
                    log::info!("Found next search index in new data: {index}");
                    return Some(index);
                }
            }
            None
        });
        if let Some(index) = index {
            self.current_match_index = Some(self.match_indices.len());
            self.match_indices.push(index);
        } else {
            self.is_end_reached = true;
        }
        index
    }

    fn find_next_index<'v, I>(query: &str, iter: I, start_index: usize) -> Option<usize>
    where
        I: Iterator<Item = &'v LogEntry>,
    {
        iter.skip(start_index)
            .enumerate()
            .find_map(|(index, entry)| entry.lower_case_message.contains(query).then_some(index))
            .map(|i| i + start_index)
    }
}
