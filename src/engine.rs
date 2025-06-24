use std::{
    env, io,
    os::unix::prelude::PermissionsExt,
    process::{Command, exit},
    time::{SystemTime, UNIX_EPOCH},
};

const SUPPORTED_LANGUAGES: [&str; 4] = ["#!shell", "#!docker", "#!python", "#!ruby"];
const DEFAULT_MESSAGE: &str = "Expected one of the following:";

use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use strsim::jaro_winkler;
use thiserror::Error;

fn isroot() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Syntax error")]
#[diagnostic(code(Error::Syntax))]
pub struct LanguageError {
    #[source_code]
    pub src: NamedSource<String>,

    #[help]
    pub help: String,

    pub label: String,

    #[label("{label}")]
    pub bad_bit: SourceSpan,
}

// #[derive(Debug, Error, Diagnostic)]
// #[error("Syntax error")]
// #[diagnostic(code(Error::Runtime))]
// struct RuntimeError {}

impl LanguageError {
    fn error(first: &str, name: &str) -> miette::Result<()> {
        match first {
            // All languages are not supported but planned for later:
            "#!node" | "#!bun" | "#!js" | "#!ts" | "#!javascript" | "#!typescript" => {
                Err(LanguageError {
                    src: NamedSource::new(name, first.to_string()),
                    help: format!("{DEFAULT_MESSAGE} {SUPPORTED_LANGUAGES:?}"),
                    label: "Javascript is not supported".to_string(),
                    bad_bit: (0, first.len()).into(),
                })?
            }
            "#!lua" => Err(LanguageError {
                src: NamedSource::new(name, first.to_string()),
                help: format!("{DEFAULT_MESSAGE} {SUPPORTED_LANGUAGES:?}"),
                label: "Lua is not supported".to_string(),
                bad_bit: (0, first.len()).into(),
            })?,
            "#!cmake" => Err(LanguageError {
                src: NamedSource::new(name, first.to_string()),
                help: format!("{DEFAULT_MESSAGE} {SUPPORTED_LANGUAGES:?}"),
                label: "cmake is not supported".to_string(),
                bad_bit: (0, first.len()).into(),
            })?,
            // Everything else:
            _ => {
                for supported in SUPPORTED_LANGUAGES {
                    if jaro_winkler(first, supported).abs() > 0.8 {
                        Err(LanguageError {
                            src: NamedSource::new(name, first.to_string()),
                            help: format!("Did you mean? {supported}"),
                            label: "Unexpected identifier".to_string(),
                            bad_bit: (0, first.len()).into(),
                        })?
                    }
                }
                Err(LanguageError {
                    src: NamedSource::new(name, first.to_string()),
                    help: format!("{DEFAULT_MESSAGE} {SUPPORTED_LANGUAGES:?}"),
                    label: "Unexpected identifier".to_string(),
                    bad_bit: (0, first.len()).into(),
                })?
            }
        }
    }
}

#[allow(dead_code)]
/// Looks at the first character of each string and checks if it is a digit, then it sorts it from
/// 0 to u32 maximum digit
fn sort(strings: &mut Vec<String>) {
    strings.retain(|s| {
        if !s.is_empty() && s.chars().next().unwrap().is_ascii_digit() {
            true
        } else {
            eprintln!("Ignoring: '{}'", s);
            false
        }
    });

    strings.sort_by_key(|s| {
        let mut end = 0;
        for (i, c) in s.char_indices() {
            if c.is_ascii_digit() {
                end = i + 1;
            } else {
                break;
            }
        }
        s[..end].parse::<u32>().unwrap_or(u32::MAX)
    });
}

pub fn shell(content: &[String], shell: &str) -> Result<(), io::Error> {
    let script = content.join("\n");
    let temp = format!(
        "/tmp/runit_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    std::fs::write(&temp, script)?;

    let mut perms = std::fs::metadata(&temp)?.permissions();
    perms.set_mode(0o755);

    Command::new(shell).arg(&temp).spawn()?.wait()?;
    std::fs::remove_file(&temp)?;
    Ok(())
}

pub fn docker(content: &[String]) -> Result<(), io::Error> {
    if !isroot() {
        eprintln!("You must be root to run docker");
    }
    let script = content.join("\n");
    let temp = "./.Dockerfile";

    std::fs::write(temp, script)?;

    let build_status = Command::new("sudo")
        .args(["docker", "build", "-t", "runit-temp", "-f", temp, "."])
        .status()?;

    if build_status.success() {
        Command::new("sudo")
            .args(["docker", "run", "--rm", "runit-temp"])
            .spawn()?
            .wait()?;
    }

    std::fs::remove_file(temp)?;
    Ok(())
}

pub fn launch(first: &str, buffer: &[String], name: &str) -> miette::Result<()> {
    match first {
        "#!shell" | "#!sh" | "#!bash" | "#!bsh" => {
            let sh = env::var("SHELL").unwrap_or_else(|_| {
                println!("No shell found, trying to use '/usr/bin/sh' instead");
                "/usr/bin/sh".to_string()
            });
            if let Err(e) = shell(buffer, &sh) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            };
        }
        "#!docker" | "#!container" => {
            // I will take the lines after #!docker and put them in a new .Dockerfile
            if let Err(e) = docker(buffer) {
                eprintln!("Error running docker ERROR({e})");
                exit(1);
            }
        }
        "#!python" | "#!py" => {
            // Python is really REALLY similar to shell execution wise so I can just modify
            // the shell function
            // I will assume the user has python installed
            let sh = "python";
            if let Err(e) = shell(buffer, sh) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            }
        }
        "#!ruby" | "#!rb" => {
            // Same situation with Ruby
            // I will assume the user has ruby installed
            let sh = "ruby";
            if let Err(e) = shell(buffer, sh) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            }
        }
        _ => LanguageError::error(first, name)?,
    }
    Ok(())
}

// Run the runits
#[allow(dead_code)]
pub fn get_directories(root: &str) -> Result<Vec<String>, io::Error> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        dirs.push(entry.file_name().to_string_lossy().to_string());
    }

    // NOTE this is a workaround because the index 0 will always be the root directory name
    dirs.remove(0);
    sort(&mut dirs);
    Ok(dirs)
}
