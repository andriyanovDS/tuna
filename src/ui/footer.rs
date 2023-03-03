use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key},
    theme::{BaseColor, ColorStyle, Effect, PaletteColor},
    view::{CannotFocus, View},
    Cursive, Printer, Vec2,
};
use std::rc::Rc;

pub type OnSubmit = dyn Fn(&mut Cursive, String);
pub struct Footer {
    search_state: SearchState,
    search_query: String,
    cursor_position: usize,
    on_submit: Rc<OnSubmit>,
    on_cancel: Rc<dyn Fn(&mut Cursive)>,
    info_color_style: ColorStyle,
    search_color_style: ColorStyle,
}

#[derive(Copy, Clone)]
enum SearchState {
    Disabled,
    Enabled,
}

impl Footer {
    pub fn new<S, C>(on_submit: S, on_cancel: C) -> Self
    where
        S: Fn(&mut Cursive, String) + 'static,
        C: Fn(&mut Cursive) + 'static,
    {
        Self {
            search_state: SearchState::Disabled,
            search_query: String::new(),
            cursor_position: 0,
            on_submit: Rc::new(on_submit),
            on_cancel: Rc::new(on_cancel),
            info_color_style: ColorStyle::new(BaseColor::Cyan, PaletteColor::Background),
            search_color_style: ColorStyle::new(BaseColor::Green, PaletteColor::Background),
        }
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
        let query = self.stop_search();
        let submit = self.on_submit.clone();
        EventResult::with_cb_once(move |c| submit(c, query))
    }

    fn stop_search(&mut self) -> String {
        self.change_cursor_position(0);
        self.search_state = SearchState::Disabled;
        std::mem::take(&mut self.search_query)
    }
}

impl View for Footer {
    fn draw(&self, printer: &Printer) {
        match self.search_state {
            SearchState::Disabled => {
                let message = "esc: cancel, q: quit, /: search";
                printer.with_color(self.info_color_style, |p| {
                    p.print((1, 0), message);
                });
            }
            SearchState::Enabled => printer.with_color(self.search_color_style, |p| {
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
        }
    }

    fn layout(&mut self, _: Vec2) {}

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        Vec2::new(constraint.x, 1)
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        match self.search_state {
            SearchState::Disabled => {
                self.search_state = SearchState::Enabled;
                Ok(EventResult::Consumed(None))
            }
            SearchState::Enabled => Err(CannotFocus),
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match self.search_state {
            SearchState::Disabled => EventResult::Ignored,
            SearchState::Enabled => match event {
                Event::Char(char) => {
                    log::info!("Char: {char}");
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
                    self.stop_search();
                    let cancel = self.on_cancel.clone();
                    EventResult::with_cb_once(move |c| cancel(c))
                }
                Event::Key(Key::Enter) => self.submit(),
                _ => EventResult::Ignored,
            },
        }
    }
}
