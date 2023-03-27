use std::{process::exit, path::Path};
use chrono::Local;
use std::io::Write;
use env_logger::{Builder, Target};
use log::LevelFilter;

fn main() {
    configure_logging();
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

fn configure_logging() {
    let log_directory_path = Path::new("/tmp/com.tuna");
    if !log_directory_path.exists() {
        std::fs::create_dir(log_directory_path).expect("Could not create log directory");
    }
    let file = std::fs::File::create(log_directory_path.join("tuna.log"))
        .expect("Could not create a log file");
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(file)))
        .filter(None, LevelFilter::Info)
        .init();
}
