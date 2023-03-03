use crate::file_reader::log_entry::LogEntry;
use crossbeam_channel::Receiver;
use cursive::{
    direction::Direction,
    event::EventResult,
    theme::{BaseColor, ColorStyle, PaletteColor, PaletteStyle, StyleType},
    view::{CannotFocus, View},
    Printer, Vec2, XY,
};

struct Styles {
    time_style: StyleType,
    source_style: StyleType,
    msg_style: StyleType,
    msg_style_hl: StyleType,
}

impl Styles {
    fn new() -> Self {
        Self {
            time_style: ColorStyle::new(BaseColor::Yellow, PaletteColor::Background).into(),
            source_style: ColorStyle::new(BaseColor::Blue, PaletteColor::Background).into(),
            msg_style: ColorStyle::new(PaletteColor::Primary, PaletteColor::Background).into(),
            msg_style_hl: PaletteStyle::Highlight.into(),
        }
    }
}

struct LogsPanelState {
    buffer: Vec<LogEntry>,
    offset: usize,
    selected_index: usize,
    receiver: Receiver<LogEntry>,
    styles: Styles,
}

impl LogsPanelState {
    fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            selected_index: 0,
            receiver,
            styles: Styles::new(),
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
        let max_y = size.y.saturating_sub(2);
        if selected_index < offset {
            state.offset = selected_index;
        } else if selected_index >= offset + max_y {
            state.offset += selected_index - offset - max_y + 1;
        }
    }

    fn draw(&self, printer: &Printer) {
        printer.print_box(Vec2::new(0, 0), printer.size, false);

        let state = &self.state;
        if state.buffer.is_empty() {
            return;
        }

        let height = printer.output_size.y.saturating_sub(2);
        let width = printer.output_size.x.saturating_sub(2);
        let mut start = state.offset;
        let end = state.buffer.len().min(start + height);
        if end - start < height {
            start = 0;
        }
        let styles = &state.styles;

        state.buffer[start..end]
            .iter()
            .enumerate()
            .for_each(|(index, entry)| {
                let y_pos = index + 1;
                let styles = if index + start == state.selected_index {
                    [styles.msg_style_hl; 3]
                } else {
                    [styles.time_style, styles.source_style, styles.msg_style]
                };
                let mut count_left = width;
                let mut start = 1;
                entry
                    .components()
                    .into_iter()
                    .zip(styles.into_iter())
                    .for_each(|(c, style)| {
                        printer.with_style(style, |p| {
                            let len = count_left.min(c.len());
                            p.print((start, y_pos), &c[..len]);
                            count_left = count_left.saturating_sub(len + 1);
                            if count_left > 0 {
                                p.print((start + len, y_pos), " ");
                            }
                            start += len + 1;
                        });
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

impl LogEntry {
    fn components(&self) -> [&str; 3] {
        match self {
            LogEntry::Empty => ["", "", ""],
            LogEntry::Info(message) => [
                &message.date_time,
                &message.source,
                &message.one_line_message,
            ],
            LogEntry::ParseFailed(error) => ["", "", &error.error_message],
        }
    }
}
