use std::fs::File;
pub mod file_reader;
pub mod ui;

pub fn handle_file(path: String) {
    let (sender, receiver) = crossbeam_channel::bounded(100);
    let mut term = ui::TermUI::new();
    let callback = term.callback().clone();
    match File::open(path) {
        Ok(file) => {
            std::thread::Builder::new()
                .name("file_processing".into())
                .spawn(move || {
                    file_reader::read_file(file, sender, callback);
                })
                .unwrap();
            term.run(receiver);
        }
        Err(error) => {
            eprintln!("Failed to open file: {error}");
        }
    }
}
