use super::{footer::Footer, state::MatchesSearchState};
use crate::file_reader::log_entry::LogEntry;
use crate::ui::state::LogsPanelState;
use crossbeam_channel::Receiver;
use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key},
    view::{CannotFocus, View},
    Printer, Vec2, XY,
};

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

    pub fn set_search_query(&mut self, query: String) -> Option<MatchesSearchState> {
        self.state.set_search_query(query);
        self.state.matches_search_state()
    }

    pub fn exit_search_mode(&mut self) {
        self.state.exit_search_mode();
    }

    fn update_pagination_state(&self) -> EventResult {
        let pagination_state = self.state.pagination_state();
        EventResult::with_cb_once(move |c| {
            c.call_on_name(Footer::name(), |view: &mut Footer| {
                view.set_pagination_state(pagination_state);
            }); 
        })
    }

    fn update_search_state(&self) -> EventResult {
        self.state
            .matches_search_state()
            .map(|state| {
                EventResult::with_cb_once(|c| {
                    c.call_on_name(Footer::name(), |view: &mut Footer| {
                        view.set_results_iteration_state(state);
                    });
                })
            })
            .unwrap_or(EventResult::Ignored)
    }
}

impl View for LogsPanel {
    fn layout(&mut self, size: XY<usize>) {
        let state = &mut self.state;
        state.load_logs(size.y);
        state.adjust_offset(size.y);
    }

    fn draw(&self, printer: &Printer) {
        printer.print_box(Vec2::new(0, 0), printer.size, false);

        let state = &self.state;
        let logs_len = state.logs_len();
        if logs_len == 0 {
            return;
        }

        let height = printer.output_size.y.saturating_sub(2);
        let width = printer.output_size.x.saturating_sub(2);
        let mut start = state.offset;
        let end = logs_len.min(start + height);
        if end - start < height {
            start = 0;
        }
        let styles = &state.styles;

        state
            .log_iter()
            .skip(start)
            .take(end - start)
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

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Up) => {
                self.state.selected_index = self.state.selected_index.saturating_sub(1);
                self.update_pagination_state()
            }
            Event::Key(Key::Down) => {
                self.state.selected_index = self
                    .state
                    .selected_index
                    .saturating_add(1)
                    .min(self.state.logs_len() - 1);
                self.update_pagination_state()
            }
            Event::Key(Key::Left) | Event::Char('j') => {
                self.state.go_to_prev_page();
                self.update_pagination_state()
            }
            Event::Key(Key::Right) | Event::Char('k') => {
                self.state.go_to_next_page();
                self.update_pagination_state()
            }
            Event::Key(Key::Esc) => {
                self.state.exit_search_mode();
                EventResult::with_cb_once(|c| {
                    c.call_on_name(Footer::name(), |view: &mut Footer| {
                        view.cancel_search();
                    });
                })
            }
            Event::Char('n') => {
                self.state.go_to_next_search_result();
                self.update_search_state()
            }
            Event::Char('N') => {
                self.state.go_to_prev_search_result();
                self.update_search_state() 
            }
            _ => EventResult::Ignored,
        }
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
