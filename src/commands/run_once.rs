use anyhow::Result;
use std::process::Command;
use crate::commands::remove;

pub fn run(id: &str, cleanup_id: &str, command: Vec<String>) -> Result<()> {
    let full_command = command.join(" ");

    // Execute the original command
    let status = Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .status()?;

    // Regardless of command success, clean up both cron entries
    let _ = remove::run(id);
    let _ = remove::run(cleanup_id);

    if !status.success() {
        let code = status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}
