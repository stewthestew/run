use console_stuff::clap::{Arg, ArgAction, command};
use miette::{NamedSource, Result};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::exit,
};

mod engine;

struct Args {
    program_name: String,
    file: String,
    runs: String,
    dry_run: bool,
    list: bool,
    init: String,
}

impl Args {
    pub fn parse() -> Self {
        let args = command!()
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .help("Path to the run file")
                    .default_value(".run"),
            )
            .arg(
                Arg::new("runs")
                    .short('r')
                    .long("runs")
                    .help(
                        "
The directory that run will recursively run
This process is called 'runs'
Runs runs files based on numeric order
The lowest number will be run first, highest last
You can start from any number 01, 0, 100, etc:
.runs/
1| 1_unit_tests # runs will run this file first 
2| 2_setup      # runs will run this file second 
3| 3_build      # runs will run this file third 
4| README.md    # Since this file does not start with a number it will be ignored 
",
                    )
                    .default_value(".runs"),
            )
            .arg(
                Arg::new("dry_run")
                    .short('d')
                    .long("dry-run")
                    .help("Prints the commands and files that will be run")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("list")
                    .short('l')
                    .long("list")
                    .help("List the files that will be run")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("init")
                    .short('i')
                    .long("init")
                    .help("Initialize a new run file\n\tChoices: simple, workflow")
                    .default_value("none"),
            );
        let name = args.get_name().to_string();
        let args = args.get_matches();
        Args {
            program_name: name,
            file: args.get_one::<String>("file").unwrap().to_owned(),
            runs: args.get_one::<String>("runs").unwrap().to_owned(),
            dry_run: args.get_flag("dry_run"),
            list: args.get_flag("list"),
            init: args.get_one::<String>("init").unwrap().to_owned(),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.init.to_lowercase().as_str() {
        "simple" => {
            if let Err(e) = init(Templates::Simple) {
                eprintln!("Error creating .run file ERROR({e})");
                exit(1);
            };
            exit(0)
        }
        "workflow" => {
            if let Err(e) = init(Templates::Workflow) {
                eprintln!("Error creating .run file ERROR({e})");
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
    if args.runs.to_lowercase() != "none" {
        runs(&args, &mut buffer)?;
    }
    Ok(())
}

// Start run
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

fn runs(args: &Args, buffer: &mut Vec<String>) -> miette::Result<()> {
    let files = match engine::get_directories(&args.runs) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    eprintln!("No {} file found", args.runs);
                    eprintln!("Did you forget to create one?");
                    eprintln!("If not do: {} -r none", args.program_name);
                }
                io::ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied to read {} file", args.file);
                }
                _ => {
                    eprintln!("Unkown error reading {} file ERROR: {e}", args.runs);
                }
            }

            exit(1);
        }
    };

    for f in files {
        let f = format!("{}/{}", args.runs, f);
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
            // Make a .run file
            // and apend to it the following
            // #!shell
            // echo "Let's run it"
            let mut file = File::create(".run")?;
            file.write_all(b"#!shell\necho \"simple template\"\n")?;
        }
        Templates::Workflow => {
            fs::create_dir(".runs")?;

            // Create example runs files
            let mut file1 = File::create(".runs/1_setup")?;
            file1.write_all(b"#!shell\necho \"Setting up project...\"\n")?;

            let mut file2 = File::create(".runs/2_build")?;
            file2.write_all(b"#!python\nprint(\"Building project...\")\n")?;

            let mut file3 = File::create(".runs/3_test")?;
            file3.write_all(b"#!shell\necho \"Running tests...\"\n")?;

            let mut file = File::create(".run")?;
            file.write_all(b"#!shell\necho \"workflow template\"\n")?;
        }
    }
    Ok(())
}
