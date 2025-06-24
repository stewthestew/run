use console_stuff::clap;
use console_stuff::prelude::Parser;
use miette::{NamedSource, Result};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::exit,
};

mod engine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(".runit"))]
    file: String,

    #[arg(short, long, default_value_t = String::from(".runits"))]
    runits: String,

    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    #[arg(short, long, alias = "ls", default_value_t = false)]
    list: bool,

    #[arg(short, long, default_value_t = String::from("none"))]
    init: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.init.to_lowercase().as_str() {
        "simple" => {
            if let Err(e) = init(Templates::Simple) {
                eprintln!("Error creating .runit file ERROR({e})");
                exit(1);
            };
            exit(0)
        }
        "workflow" => {
            if let Err(e) = init(Templates::Workflow) {
                eprintln!("Error creating .runit file ERROR({e})");
            };
            exit(0)
        }
        "none" => {}
        _ => {
            eprintln!("Unknown template");
            exit(1);
        }
    }

    let mut buffer: Vec<String> = Vec::new();
    start(&args, &mut buffer)?;
    if args.runits.to_lowercase() != "none" {
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

    if !args.file.is_empty() && !args.dry_run && !args.list {
        engine::launch(first, buffer, &args.file)?;
    } else if args.list {
        println!("{}", args.file);
    } else if args.dry_run {
        println!("\n ╭─{}", args.file);
        for (i, b) in buffer.iter().enumerate() {
            println!("{}│ {}", i + 1, b);
        }
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

        if !f.is_empty() && !args.dry_run && !args.list {
            engine::launch(first, buffer, &f)?;
        } else if args.list {
            println!("{}", f);
        } else if args.dry_run {
            println!("\n ╭─{}", f);
            for (i, b) in buffer.iter().enumerate() {
                println!("{}│ {}", i + 1, b);
            }
        }
    }

    Ok(())
}

enum Templates {
    Simple,
    Workflow,
}

// Hardcoded templates for now
// Might not do that later?
fn init(template: Templates) -> io::Result<()> {
    match template {
        Templates::Simple => {
            // Make a .runit file
            // and apend to it the following
            // #!shell
            // echo "Let's run it"
            let mut file = File::create(".runit")?;
            file.write_all(b"#!shell\necho \"simple template\"\n")?;
        }
        Templates::Workflow => {
            fs::create_dir(".runits")?;

            // Create example runits files
            let mut file1 = File::create(".runits/1_setup")?;
            file1.write_all(b"#!shell\necho \"Setting up project...\"\n")?;

            let mut file2 = File::create(".runits/2_build")?;
            file2.write_all(b"#!python\nprint(\"Building project...\")\n")?;

            let mut file3 = File::create(".runits/3_test")?;
            file3.write_all(b"#!shell\necho \"Running tests...\"\n")?;

            let mut file = File::create(".runit")?;
            file.write_all(b"#!shell\necho \"workflow template\"\n")?;
        }
    }
    Ok(())
}
