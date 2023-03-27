use super::data_source::PaginationState;
use super::{data_source::SearchPaginationState, dialog_content::DialogContent, footer::Footer};
use crate::file_reader::log_entry::LogEntry;
use crate::ui::data_source::DataSource;
use crossbeam_channel::Receiver;
use cursive::theme::{BaseColor, ColorStyle, PaletteColor, PaletteStyle, StyleType};
use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key},
    view::{CannotFocus, View},
    views::{Checkbox, ListView},
    Printer, Vec2, XY,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub struct Styles {
    pub time_style: StyleType,
    pub source_style: StyleType,
    pub msg_style: StyleType,
    pub msg_style_hl: StyleType,
    pub lines_style: StyleType,
}

impl Styles {
    pub fn new() -> Self {
        Self {
            time_style: ColorStyle::new(BaseColor::Yellow, PaletteColor::Background).into(),
            source_style: ColorStyle::new(BaseColor::Blue, PaletteColor::Background).into(),
            msg_style: ColorStyle::new(PaletteColor::Primary, PaletteColor::Background).into(),
            msg_style_hl: PaletteStyle::Highlight.into(),
            lines_style: ColorStyle::new(BaseColor::Cyan, PaletteColor::Background).into(),
        }
    }
}

pub struct LogsPanel {
    state: DataSource,
    styles: Styles,
}

impl LogsPanel {
    pub fn new(receiver: Receiver<LogEntry>) -> Self {
        Self {
            state: DataSource::new(receiver),
            styles: Styles::new(),
        }
    }

    pub fn name() -> &'static str {
        "logs_panel"
    }

    pub fn set_search_query(&mut self, query: String) -> SearchPaginationState {
        self.state.start_search(query);
        self.state.search_pagination_state()
    }

    pub fn exit_search_mode(&mut self) {
        self.state.stop_search();
    }

    pub fn set_selected_sources(&mut self, sources: HashSet<u64>) -> PaginationState {
        self.state.set_selected_sources(sources);
        self.state.pagination_state()
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
        let state = self.state.search_pagination_state();
        EventResult::with_cb_once(|c| {
            c.call_on_name(Footer::name(), |view: &mut Footer| {
                view.set_results_iteration_state(state);
            });
        })
    }

    fn show_active_message(&self) -> EventResult {
        let entry = self.state.active_message().unwrap().clone();
        EventResult::with_cb_once(|c| {
            let content = DialogContent::new(entry);
            let dialog = cursive::views::Dialog::around(content)
                .title("Message")
                .dismiss_button("Close");
            c.add_layer(dialog);
        })
    }

    fn show_source_filter(&self) -> EventResult {
        let mut list_view = ListView::new();
        let selected = Rc::new(RefCell::new(std::collections::HashSet::<u64>::new()));
        self.state.iterate_sources(|(source, is_selected)| {
            let hash = source.hash;
            if is_selected {
                selected.as_ref().borrow_mut().insert(hash);
            }
            let selected = selected.clone();
            let mut checkbox = Checkbox::new().on_change(move |_, is_selected| {
                let mut selected = selected.as_ref().borrow_mut();
                if is_selected {
                    selected.insert(hash);
                } else {
                    selected.remove(&hash);
                }
            });
            checkbox.set_checked(is_selected);
            list_view.add_child(&source.name, checkbox);
        });
        EventResult::with_cb_once(|c| {
            let dialog = cursive::views::Dialog::new()
                .title("Sources")
                .dismiss_button("Close")
                .content(list_view)
                .button("Submit", move |c| {
                    let pagination_state = c.call_on_name(Self::name(), |view: &mut LogsPanel| {
                        let selected = selected.as_ref().replace(HashSet::new());
                        view.set_selected_sources(selected)
                    });
                    c.call_on_name(Footer::name(), |view: &mut Footer| {
                        view.set_pagination_state(pagination_state.unwrap())
                    });
                    c.pop_layer();
                });
            c.add_layer(dialog);
        })
    }
}

impl View for LogsPanel {
    fn layout(&mut self, size: XY<usize>) {
        let state = &mut self.state;
        state.load_logs(size.y);
        state.prepare_for_draw(size.y.saturating_sub(2));
    }

    fn draw(&self, printer: &Printer) {
        printer.print_box(Vec2::new(0, 0), printer.size, false);

        let styles = &self.styles;
        let width = printer.output_size.x.saturating_sub(2);
        let selected_index = self.state.selected_index - self.state.offset;

        self.state.iterate_entries_to_draw(|(index, entry)| {
            let y_pos = index + 1;
            let components_styles = if index == selected_index {
                [styles.msg_style_hl; 3]
            } else {
                [styles.time_style, styles.source_style, styles.msg_style]
            };
            let lines = if entry.lines_count > 1 {
                format!("[+{} lines]", entry.lines_count - 1)
            } else {
                String::new()
            };
            let mut count_left = width.saturating_sub(lines.len() + 1);
            let mut start = 1;
            let components = [
                &entry.date_time,
                &entry.source.name,
                &entry.one_line_message,
            ];
            components
                .into_iter()
                .zip(components_styles.into_iter())
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
            if !lines.is_empty() {
                printer.with_style(styles.lines_style, |p| {
                    p.print((width.saturating_sub(lines.len() - 1), y_pos), &lines);
                })
            }
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
                self.state.select_previous();
                self.update_pagination_state()
            }
            Event::Key(Key::Down) => {
                self.state.select_next();
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
                self.state.stop_search();
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
            Event::Char('s') => self.show_source_filter(),
            Event::Key(Key::Enter) => self.show_active_message(),
            _ => EventResult::Ignored,
        }
    }
}
