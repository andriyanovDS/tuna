pub mod file_reader;

pub fn handle_file(path: String) {
    match file_reader::read_file(path) {
        Ok(iter) => {
            render_entries(iter);
        }
        Err(error) => {
            eprintln!("Failed to open file {error}");
        }
    }
}

fn render_entries(iter: impl Iterator<Item=file_reader::LogEntry>) {
    for entry in iter {
        
    }
}
