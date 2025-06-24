use console_stuff::clap;
use console_stuff::prelude::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    process::exit,
    time::Duration,
};

use console_stuff::prelude::ProgressBar;

mod engine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(".runit"))]
    file: String,
}

fn main() {
    let args = Args::parse();
    let argfile = &args.file;
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));

    let mut buffer: Vec<String> = Vec::new();
    bar.set_message(format!("Opening {argfile} file"));
    let file = match File::open(argfile) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    eprintln!("No {argfile} file found");
                    eprintln!("Did you forget to create one?");
                }
                io::ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied to read {argfile} file");
                }
                _ => {
                    eprintln!("Unknown error reading {argfile} file ERROR({e})");
                }
            };
            exit(1)
        }
    };
    bar.set_message("Reading file");
    let reader = BufReader::new(file);
    for line in reader.lines() {
        buffer.push(line.unwrap_or_else(|e| {
            eprintln!("Error reading {argfile} file ERROR({e})");
            exit(1);
        }));
    }

    bar.set_message("Getting the identifier");
    let first = buffer
        .first()
        .unwrap_or_else(|| {
            eprintln!("Error reading {argfile} file");
            exit(1);
        })
        .as_str();

    // NOTE Before I do anything I check if the buffer is empty
    if buffer.is_empty() {
        eprintln!("Empty {argfile} file");
        // I won't exit since this isn't really an erorr just stupidity
    }

    bar.set_message("Launching");
    engine::launch(first, &buffer);
}
