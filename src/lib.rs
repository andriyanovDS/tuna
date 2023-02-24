use std::fs::File;
use std::sync::mpsc;

pub mod file_reader;
pub mod ui;

pub fn handle_file(path: String) {
    let (sender, receiver) = mpsc::channel();
    let mut term = ui::TermUI::new();
    let callback = term.callback().clone();
    match File::open(path) {
        Ok(file) => {
            std::thread::spawn(move || {
                file_reader::read_file(file, sender, callback);
            });
            term.run(receiver);
        }
        Err(error) => {
            eprintln!("Failed to open file: {error}");
        }
    }
}
