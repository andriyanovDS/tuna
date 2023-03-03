use crate::file_reader::log_entry::LogEntry;
use crossbeam_channel::Receiver;
use cursive::{
    direction::Direction,
    event::EventResult,
    theme::{PaletteStyle, Style, StyleType},
    view::{CannotFocus, View},
    Printer, Vec2, XY,
};

struct LogsPanelState {
    buffer: Vec<LogEntry>,
    offset: usize,
    selected_index: usize,
    receiver: Receiver<LogEntry>,
}

impl LogsPanelState {
    fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            selected_index: 0,
            receiver,
        }
    }
}
pub struct LogsPanel {
    state: LogsPanelState,
}

impl LogsPanel {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            state: LogsPanelState::new(receiver),
        }
    }

    pub fn name() -> &'static str {
        "logs_panel"
    }

    pub fn set_search_query(&mut self, query: String) {
        log::info!("Receive query {query}");
    }

    pub fn select_next(&mut self) {
        self.state.selected_index = self
            .state
            .selected_index
            .saturating_add(1)
            .min(self.state.buffer.len() - 1);
    }

    pub fn select_prev(&mut self) {
        self.state.selected_index = self.state.selected_index.saturating_sub(1);
    }
}

impl View for LogsPanel {
    fn layout(&mut self, size: XY<usize>) {
        let state = &mut self.state;
        let offset = state.offset;
        let diff = (offset + size.y * 2).saturating_sub(state.buffer.len());
        let mut request_count = diff;
        while request_count > 0 {
            if let Ok(entry) = state.receiver.recv() {
                state.buffer.push(entry);
                request_count -= 1;
            } else {
                request_count = 0;
            }
        }
        let selected_index = state.selected_index;
        if selected_index < offset {
            state.offset = selected_index;
        } else if selected_index >= offset + size.y {
            state.offset += selected_index - offset - size.y + 1;
        }
    }

    fn draw(&self, printer: &Printer) {
        let state = &self.state;
        if state.buffer.is_empty() {
            return;
        }
        let regular_style: StyleType = Style::inherit_parent().into();
        let highlight_style: StyleType = PaletteStyle::Highlight.into();

        let height = printer.output_size.y;
        let mut start = state.offset;
        let end = state.buffer.len().min(start + height);
        if end - start < height {
            start = 0;
        }

        state.buffer[start..end]
            .iter()
            .enumerate()
            .for_each(|(index, entry)| {
                let style = if index + start == state.selected_index {
                    highlight_style
                } else {
                    regular_style
                };
                printer.with_style(style, |printer| {
                    printer.print((0, index), entry.display());
                });
            });
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        XY::new(constraint.x, constraint.y - 1)
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }
}
