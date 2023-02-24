use crate::file_reader::LogEntry;
use cursive::{
    traits::Scrollable,
    view::{self, Resizable},
    views, CbSink, CursiveRunnable, Vec2,
};
use std::sync::mpsc::Receiver;

struct State {
    buffer: Vec<LogEntry>,
    receiver: Receiver<LogEntry>,
}

impl State {
    fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            buffer: Vec::new(),
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
        self.runnable.add_layer(TermUI::build_ui(state));
        self.runnable.add_global_callback('q', |c| c.quit());
        self.runnable.run();
    }

    fn build_ui(state: State) -> impl view::View {
        views::LinearLayout::vertical()
            .child(TermUI::build_logs_view(state))
            .full_screen()
    }

    fn build_logs_view(state: State) -> impl view::View {
        views::Canvas::new(state)
            .with_layout(|state, _| {
                while let Ok(entry) = state.receiver.recv() {
                    state.buffer.push(entry);
                }
            })
            .with_draw(|state, printer| {
                state
                    .buffer
                    .iter()
                    .enumerate()
                    .for_each(|(index, entry)| printer.print((0, index), &entry.display()));
            })
            .with_required_size(|state, _| Vec2::new(20, state.buffer.len()))
            .scrollable()
    }
}

impl LogEntry {
    fn display(&self) -> String {
        match self {
            LogEntry::Empty => String::from(" "),
            LogEntry::Info(info) => info.clone(),
        }
    }
}
