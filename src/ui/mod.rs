use crate::file_reader::log_entry::LogEntry;
use cursive::{
    event::{EventResult, Key},
    theme::Theme,
    theme::{PaletteStyle, Style, StyleType},
    view::{self, Resizable},
    views::{self, OnEventView},
    CbSink, CursiveRunnable, Printer, XY,
};
use std::sync::mpsc::Receiver;

struct State {
    buffer: Vec<LogEntry>,
    offset: usize,
    selected_index: usize,
    receiver: Receiver<LogEntry>,
}

impl State {
    fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            selected_index: 0,
            receiver,
        }
    }
}

pub struct TermUI {
    runnable: CursiveRunnable,
}

impl TermUI {
    pub fn new() -> Self {
        Self {
            runnable: cursive::default(),
        }
    }

    pub fn callback(&self) -> &CbSink {
        self.runnable.cb_sink()
    }

    pub fn run(&mut self, receiver: Receiver<LogEntry>) {
        let state = State::new(receiver);
        self.runnable.set_theme(Theme::terminal_default());
        self.runnable.add_layer(TermUI::build_ui(state));
        self.runnable.add_global_callback('q', |c| c.quit());
        self.runnable
            .add_global_callback('d', |c| c.toggle_debug_console());
        cursive::logger::init();
        self.runnable.run();
    }

    fn build_ui(state: State) -> impl view::View {
        views::LinearLayout::vertical()
            .child(TermUI::build_logs_view(state))
            .full_screen()
    }

    fn build_logs_view(state: State) -> impl view::View {
        let canvas = views::Canvas::new(state)
            .with_layout(TermUI::layout)
            .with_draw(TermUI::draw)
            .with_required_size(|_, size| size);

        OnEventView::new(canvas)
            .on_pre_event_inner(Key::Up, |inner, _| {
                let state = inner.state_mut();
                state.selected_index = state.selected_index.saturating_sub(1);
                Some(EventResult::Consumed(None))
            })
            .on_pre_event_inner(Key::Down, |inner, _| {
                let state = inner.state_mut();
                state.selected_index = state
                    .selected_index
                    .saturating_add(1)
                    .min(state.buffer.len() - 1);
                Some(EventResult::Consumed(None))
            })
    }

    fn layout(state: &mut State, size: XY<usize>) {
        while let Ok(entry) = state.receiver.recv() {
            state.buffer.push(entry);
        }
        let selected_index = state.selected_index;
        let offset = state.offset;
        if selected_index < offset {
            state.offset = selected_index;
        } else if selected_index >= offset + size.y {
            state.offset += selected_index - offset - size.y + 1;
        }
    }

    fn draw(state: &State, printer: &Printer) {
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
                    printer.print((0, index), &entry.display());
                });
            });
    }
}

impl LogEntry {
    fn display(&self) -> String {
        match self {
            LogEntry::Empty => String::from(" "),
            LogEntry::Info(message) => {
                format!(
                    "{} [{}] {}",
                    message.date_time, message.source, message.message
                )
            }
        }
    }
}
