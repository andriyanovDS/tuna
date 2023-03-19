use super::state::Styles;
use crate::file_reader::log_entry::LogEntry;
use cursive::{view::View, Vec2};
use itertools::Itertools;

pub struct DialogContent {
    message: LogEntry,
    date_full: String,
    required_size: Option<Vec2>,
    styles: Styles,
}

impl DialogContent {
    pub fn new(message: LogEntry) -> Self {
        Self {
            message,
            date_full: String::new(),
            required_size: None,
            styles: Styles::new(),
        }
    }
}

impl View for DialogContent {
    fn layout(&mut self, _: Vec2) {
        if self.date_full.is_empty() {
            self.date_full = self.message.date_full();
        }
    }

    fn draw(&self, printer: &cursive::Printer) {
        printer.with_style(self.styles.time_style, |p| {
            p.print((1, 0), &self.date_full);
        });
        printer.with_style(self.styles.source_style, |p| {
            p.print((1, 1), &self.message.source.name);
        });
        printer.with_style(self.styles.msg_style, |p| {
            let width = printer.output_size.x;
            let mut y_pos = 2;
            self.message.message.lines().for_each(|line| {
                line.chars()
                    .chunks(width - 2)
                    .into_iter()
                    .map(|chunk| chunk.collect::<String>())
                    .for_each(|line| {
                        p.print((1, y_pos), &line);
                        y_pos += 1;
                    })
            });
        });
    }

    fn required_size(&mut self, size: Vec2) -> Vec2 {
        match self.required_size {
            Some(size) => size,
            None => {
                let max_width = size.x;
                let size = self
                    .message
                    .message
                    .lines()
                    .fold(Vec2::new(1, 2), |size, line| {
                        let lines_count = line.chars().chunks(max_width - 2).into_iter().count();
                        let width = if lines_count > 1 {
                            max_width
                        } else {
                            line.len() + 2
                        };
                        Vec2::new(size.x.max(width), size.y + lines_count)
                    });
                self.required_size = Some(size);
                size
            }
        }
    }
}
