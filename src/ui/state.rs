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
    fn new() -> Self {
        Self {
            time_style: ColorStyle::new(BaseColor::Yellow, PaletteColor::Background).into(),
            source_style: ColorStyle::new(BaseColor::Blue, PaletteColor::Background).into(),
            msg_style: ColorStyle::new(PaletteColor::Primary, PaletteColor::Background).into(),
            msg_style_hl: PaletteStyle::Highlight.into(),
        }
    }
}

pub struct LogsPanelState {
    pub offset: usize,
    pub selected_index: usize,
    pub styles: Styles,
    buffer: Vec<LogEntry>,
    receiver: Receiver<LogEntry>,
}

impl LogsPanelState {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            selected_index: 0,
            receiver,
            styles: Styles::new(),
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
        let selected_index = self.selected_index;
        let max_y = screen_height.saturating_sub(2);
        let offset = self.offset;
        if selected_index < offset {
            self.offset = selected_index;
        } else if selected_index >= offset + max_y {
            self.offset += selected_index - offset - max_y + 1;
        }
    }
}