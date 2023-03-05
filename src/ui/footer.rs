use super::logs_panel::LogsPanel;
use super::state::MatchesSearchState;
use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key},
    theme::{BaseColor, ColorStyle, Effect, PaletteColor},
    view::{CannotFocus, View},
    Printer, Vec2,
};

pub struct Footer {
    search_state: SearchState,
    search_query: String,
    cursor_position: usize,
    info_color_style: ColorStyle,
    search_color_style: ColorStyle,
}

enum SearchState {
    Disabled,
    Input,
    ResultsIteration(MatchesSearchState),
}

impl Footer {
    pub fn new() -> Self {
        Self {
            search_state: SearchState::Disabled,
            search_query: String::new(),
            cursor_position: 0,
            info_color_style: ColorStyle::new(BaseColor::Cyan, PaletteColor::Background),
            search_color_style: ColorStyle::new(BaseColor::Green, PaletteColor::Background),
        }
    }

    pub fn name() -> &'static str {
        "footer_view"
    }

    pub fn cancel_search(&mut self) {
        self.change_cursor_position(0);
        self.search_state = SearchState::Disabled;
        self.search_query = String::new();
    }

    pub fn set_results_iteration_state(&mut self, state: MatchesSearchState) {
        self.search_state = SearchState::ResultsIteration(state);
    }

    fn insert(&mut self, character: char) {
        self.search_query.push(character);
        self.cursor_position += character.len_utf8();
    }

    fn delete_last(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let char = self.search_query.remove(self.search_query.len() - 1);
        self.cursor_position -= char.len_utf8();
    }

    fn change_cursor_position(&mut self, position: usize) {
        self.cursor_position = position;
    }

    fn submit(&mut self) -> EventResult {
        self.change_cursor_position(0);
        let query = self.search_query.clone();
        EventResult::with_cb_once(|c| {
            let mut res = None;
            c.call_on_name(LogsPanel::name(), |view: &mut LogsPanel| {
                res = view.set_search_query(query);
            });
            c.focus_name(LogsPanel::name()).unwrap();
            let Some(state) = res else {
                return;
            };
            c.call_on_name(Footer::name(), |view: &mut Footer| {
                view.set_results_iteration_state(state)
            });
        })
    }
}

impl View for Footer {
    fn draw(&self, printer: &Printer) {
        match &self.search_state {
            SearchState::Disabled => {
                let message = "esc: cancel, q: quit, /: search";
                printer.with_color(self.info_color_style, |p| {
                    p.print((1, 0), message);
                });
            }
            SearchState::Input => printer.with_color(self.search_color_style, |p| {
                let search_msg = "search: ";
                p.print((1, 0), search_msg);
                p.print((search_msg.len() + 1, 0), &self.search_query);

                let cursor_position = search_msg.len() + self.cursor_position + 1;
                p.with_effect(Effect::Reverse, |p| {
                    let position = self.cursor_position;
                    if position < self.search_query.len() {
                        let char = &self.search_query[position..position + 1];
                        p.print((cursor_position, 0), char);
                    } else {
                        p.print((cursor_position, 0), " ");
                    }
                })
            }),
            SearchState::ResultsIteration(MatchesSearchState::NoMatchesFound) => {
                let mut start_pos = 1;
                printer.with_color(self.search_color_style, |p| {
                    ["search: no matches for '", &self.search_query, "'"]
                        .into_iter()
                        .for_each(|m| {
                            p.print((start_pos, 0), m);
                            start_pos += m.len();
                        });
                });
                printer.with_color(self.info_color_style, |p| {
                    p.print((start_pos + 1, 0), "esc: exit search mode");
                });
            }
            SearchState::ResultsIteration(MatchesSearchState::MatchesIteration(s)) => {
                let mut start_pos = 1;
                let current = s.current;
                printer.with_color(self.search_color_style, |p| {
                    let page_msg = s
                        .total
                        .map(|total| format!("({current} of {total})"))
                        .unwrap_or_else(|| format!("({current} of ?)"));
                    ["search: matches for '", &self.search_query, "' ", &page_msg]
                        .into_iter()
                        .for_each(|m| {
                            p.print((start_pos, 0), m);
                            start_pos += m.len();
                        });
                });
                printer.with_color(self.info_color_style, |p| {
                    [
                        "n: next match, ",
                        "N: previous match, ",
                        "esc: exit search mode",
                    ]
                    .into_iter()
                    .for_each(|m| {
                        p.print((start_pos + 1, 0), m);
                        start_pos += m.len();
                    });
                });
            }
        }
    }

    fn layout(&mut self, _: Vec2) {}

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Vec2::new(constraint.x, 1)
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        match self.search_state {
            SearchState::Disabled | SearchState::ResultsIteration(_) => {
                self.search_state = SearchState::Input;
                self.search_query = String::new();
                Ok(EventResult::Consumed(None))
            }
            SearchState::Input => Err(CannotFocus),
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match self.search_state {
            SearchState::Disabled | SearchState::ResultsIteration(_) => EventResult::Ignored,
            SearchState::Input => match event {
                Event::Char(char) => {
                    self.insert(char);
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Backspace) => {
                    self.delete_last();
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Left) if self.cursor_position > 0 => {
                    self.change_cursor_position(self.cursor_position - 1);
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Right) => {
                    let position = self.search_query.len().min(self.cursor_position + 1);
                    self.change_cursor_position(position);
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Esc) => {
                    self.cancel_search();
                    EventResult::with_cb_once(move |c| {
                        c.call_on_name(LogsPanel::name(), |v: &mut LogsPanel| {
                            v.exit_search_mode();
                        });
                        c.focus_name(LogsPanel::name()).unwrap();
                    })
                }
                Event::Key(Key::Enter) => self.submit(),
                _ => EventResult::Ignored,
            },
        }
    }
}
