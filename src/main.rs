use console_stuff::clap;
use console_stuff::prelude::Parser;
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    process::exit,
    time::Duration,
};

use console_stuff::prelude::ProgressBar;

mod launch;

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

    bar.set_message("Checking identifier");
    match first {
        "#!shell" => {
            bar.set_message("Running");
            let shell = env::var("SHELL").unwrap_or_else(|_| {
                println!("No shell found, trying to use '/usr/bin/sh' instead");
                "/usr/bin/sh".to_string()
            });
            bar.finish();
            if let Err(e) = launch::shell(&buffer, &shell) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            };
        }
        "#!docker" => {
            // I will take the lines after #!docker and put them in a new .Dockerfile
            bar.set_message("Running");
            bar.finish();
            if let Err(e) = launch::docker(&buffer) {
                eprintln!("Error running docker ERROR({e})");
                exit(1);
            }
        }
        "#!python" | "#!py" => {
            // Python is really REALLY similar to shell execution wise so I can just modify
            // the shell function
            // I will assume the user has python installed
            bar.set_message("Running");
            bar.finish();
            let shell = "python";
            if let Err(e) = launch::shell(&buffer, shell) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            }
        }
        _ => {
            eprintln!("Unknown identifier.\nExpected:[#!shell, #!docker, #!python]\nFound:{first}");
        }
    }
}
