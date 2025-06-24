use console_stuff::clap;
use console_stuff::prelude::Parser;
use miette::{NamedSource, Result};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    process::exit,
};

mod engine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(".runit"))]
    file: String,

    #[arg(short, long, default_value_t = String::new())]
    runits: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut buffer: Vec<String> = Vec::new();
    start(&args, &mut buffer)?;
    // NOTE I think runits already clears the buffer, but just incase.
    if !args.runits.is_empty() {
        runits(&args, &mut buffer)?;
    }
    Ok(())
}

// Start runit
fn start(args: &Args, buffer: &mut Vec<String>) -> miette::Result<()> {
    let file = match File::open(&args.file) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    eprintln!("No {} file found", args.file);
                    eprintln!("Did you forget to create one?");
                }
                io::ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied to read {} file", args.file);
                }
                _ => {
                    eprintln!("Unknown error reading {} file ERROR({e})", args.file);
                }
            };
            exit(1)
        }
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        buffer.push(line.unwrap_or_else(|e| {
            eprintln!("Error reading {} file ERROR({e})", args.file);
            exit(1);
        }));
    }

    let first = buffer.first();

    let first = match first {
        Some(s) => s,
        None => "",
    };

    // NOTE Before I do anything I check if the buffer is empty
    if buffer.is_empty() {
        Err(engine::LanguageError {
            src: NamedSource::new(&args.file, " ".to_string()),
            help: "Add an identifier to the first line of the file".to_string(),
            label: "Empty file".to_string(),
            bad_bit: (0, 1).into(),
        })?;
    }

    if !args.file.is_empty() {
        engine::launch(first, buffer, &args.file)?;
    }

    Ok(())
}

fn runits(args: &Args, buffer: &mut Vec<String>) -> miette::Result<()> {
    let files = match engine::get_directories(&args.runits) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error reading {} file ERROR({e})", args.runits);
            exit(1);
        }
    };

    for f in files {
        let f = format!("{}/{}", args.runits, f);
        buffer.clear();
        let file = match File::open(&f) {
            Ok(f) => f,
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        eprintln!("No {} file found", f);
                        eprintln!("Did you forget to create one?");
                    }
                    io::ErrorKind::PermissionDenied => {
                        eprintln!("Permission denied to read {} file", f);
                    }
                    _ => {
                        eprintln!("Unknown error reading {} file ERROR({e})", f);
                    }
                };
                exit(1)
            }
        };

        let reader = BufReader::new(file);
        for line in reader.lines() {
            buffer.push(line.unwrap_or_else(|e| {
                eprintln!("Error reading {} file ERROR({e})", f);
                exit(1);
            }));
        }

        let first = buffer.first();

        let first = match first {
            Some(s) => s,
            None => "",
        };

        // NOTE Before I do anything I check if the buffer is empty
        if buffer.is_empty() {
            Err(engine::LanguageError {
                src: NamedSource::new(&f, " ".to_string()),
                help: "Add an identifier to the first line of the file".to_string(),
                label: "Empty file".to_string(),
                bad_bit: (0, 1).into(),
            })?;
        }

        if !f.is_empty() {
            engine::launch(first, buffer, &f)?;
        }
    }

    Ok(())
}
