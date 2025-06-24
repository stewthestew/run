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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let argfile = &args.file;

    let mut buffer: Vec<String> = Vec::new();
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

    let reader = BufReader::new(file);
    for line in reader.lines() {
        buffer.push(line.unwrap_or_else(|e| {
            eprintln!("Error reading {argfile} file ERROR({e})");
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
            src: NamedSource::new(argfile, " ".to_string()),
            help: "Add an identifier to the first line of the file".to_string(),
            label: "Empty file".to_string(),
            bad_bit: (0, 1).into(),
        })?;
    }

    engine::launch(first, &buffer, argfile)?;
    Ok(())
}
