use std::{
    env, io,
    os::unix::prelude::PermissionsExt,
    process::{Command, exit},
    time::{SystemTime, UNIX_EPOCH},
};

fn isroot() -> bool {
    unsafe { libc::geteuid() == 0 }
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
        exit(1);
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

pub fn launch(first: &str, buffer: &[String]) {
    match first {
        "#!shell" => {
            let sh = env::var("SHELL").unwrap_or_else(|_| {
                println!("No shell found, trying to use '/usr/bin/sh' instead");
                "/usr/bin/sh".to_string()
            });
            if let Err(e) = shell(buffer, &sh) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            };
        }
        "#!docker" => {
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
        "#!ruby" => {
            // Python is really REALLY similar to shell execution wise so I can just modify
            // the shell function
            // I will assume the user has ruby installed
            let sh = "ruby";
            if let Err(e) = shell(buffer, sh) {
                eprintln!("Error running shell ERROR({e})");
                exit(1);
            }
        }
        _ => {
            eprintln!("Unknown identifier.\nExpected:[#!shell, #!docker, #!python]\nFound:{first}");
        }
    }
}

// Run the runits
#[allow(dead_code)]
pub fn runits(root: &str) -> Result<Vec<String>, io::Error> {
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
