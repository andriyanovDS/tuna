use crate::file_reader::log_entry::LogEntry;
use crossbeam_channel::Receiver;
use cursive::{
    event::{EventResult, Key},
    theme::Theme,
    view::{self, Nameable, Resizable},
    views::{self, OnEventView},
    CbSink, CursiveRunnable,
};
use logs_panel::LogsPanel;

mod footer;
mod logs_panel;
mod state;

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
        self.runnable.set_theme(Theme::terminal_default());
        self.runnable.set_window_title("Tuna");
        self.runnable
            .add_fullscreen_layer(TermUI::build_ui(receiver));
        self.runnable.add_global_callback('q', |c| c.quit());
        self.runnable
            .add_global_callback('d', |c| c.toggle_debug_console());
        cursive::logger::init();
        self.runnable.run();
    }

    fn build_ui(receiver: Receiver<LogEntry>) -> impl view::View {
        let footer_view = footer::Footer::new(
            |c, query| {
                c.call_on_name(LogsPanel::name(), |view: &mut LogsPanel| {
                    view.set_search_query(query);
                });
                c.focus_name(LogsPanel::name()).unwrap();
            },
            |c| {
                c.focus_name(LogsPanel::name()).unwrap();
            },
        );
        let view = views::LinearLayout::vertical()
            .child(TermUI::build_logs_view(receiver))
            .child(footer_view)
            .full_screen();

        OnEventView::new(view).on_pre_event_inner('/', |inner, _| {
            let inner = inner.get_inner_mut();
            if inner.get_focus_index() == 1 {
                None
            } else {
                inner
                    .set_focus_index(1)
                    .map(|_| EventResult::Consumed(None))
                    .ok()
            }
        })
    }

    fn build_logs_view(receiver: Receiver<LogEntry>) -> impl view::View {
        let view = LogsPanel::new(receiver).with_name(LogsPanel::name());
        OnEventView::new(view)
            .on_pre_event_inner(Key::Up, |inner, _| {
                inner.get_mut().select_prev();
                Some(EventResult::Consumed(None))
            })
            .on_pre_event_inner(Key::Down, |inner, _| {
                inner.get_mut().select_next();
                Some(EventResult::Consumed(None))
            })
    }
}
