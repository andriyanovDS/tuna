use std::process::exit;

use tuna;

fn main() {
    let file_path = std::env::args().skip(1).next();
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
