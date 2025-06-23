use std::{
    io,
    os::unix::prelude::PermissionsExt,
    process::{Command, exit},
    time::{SystemTime, UNIX_EPOCH},
};

fn isroot() -> bool {
    unsafe { libc::geteuid() == 0 }
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
