use std::process::exit;

fn main() {
    let file_path = std::env::args().nth(1);
    match file_path {
        Some(path) => {
            tuna::handle_file(path);
        }
        None => {
            println!("Usage: tuna <path to log file>");
            exit(1);
        }
    }
}
