use std::process::exit;

use tuna;

fn main() {
    let file_path = std::env::args().skip(1).next();
    match file_path {
        Some(path) => {
            print!("{path} path provided");
            tuna::handle_file(path);
        }
        None => {
            print!("Usage: tuna <path to log file>");
            exit(1);
        }
    }
}
