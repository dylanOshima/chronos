use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AtEntry {
    pub job_number: u32,
    pub scheduled_time: String,
    pub command: Option<String>,
}

pub fn parse_atq_output(output: &str) -> Vec<AtEntry> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let job_number: u32 = parts.next()?.trim().parse().ok()?;
            let scheduled_time = parts.next()?.to_string();
            Some(AtEntry {
                job_number,
                scheduled_time,
                command: None,
            })
        })
        .collect()
}

#[allow(dead_code)]
pub fn parse_at_availability_check(success: bool) -> bool {
    success
}

pub fn is_at_available() -> bool {
    let has_binary = Command::new("which")
        .arg("at")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_binary {
        return false;
    }

    // On macOS, `at` exists but the daemon (atrun) is disabled by default.
    // Check that it's actually loaded before declaring `at` usable.
    // `launchctl list` only sees the per-user domain, where system daemons
    // never appear — query the system domain via `print`, which works
    // without sudo and succeeds only when atrun is bootstrapped.
    if cfg!(target_os = "macos") {
        return Command::new("launchctl")
            .args(["print", "system/com.apple.atrun"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }

    true
}

pub fn extract_command_from_at_script(script: &str) -> String {
    script
        .lines()
        .filter(|line| {
            !line.starts_with("#!")
                && !line.starts_with('#')
                && !line.starts_with("export ")
                && !line.is_empty()
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

pub fn read_at_queue() -> Result<Vec<AtEntry>> {
    let output = Command::new("atq").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = parse_atq_output(&stdout);

    for entry in &mut entries {
        let script_output = Command::new("at")
            .arg("-c")
            .arg(entry.job_number.to_string())
            .output()?;
        let script = String::from_utf8_lossy(&script_output.stdout);
        let command = extract_command_from_at_script(&script);
        if !command.is_empty() {
            entry.command = Some(command);
        }
    }

    Ok(entries)
}

/// Convert the internal "HH:MM YYYY-MM-DD" one-off time into the POSIX
/// `at -t` operand ([[CC]YY]MMDDhhmm). BSD/macOS `at` rejects the natural
/// "HH:MM YYYY-MM-DD" form that GNU at accepts, so `-t` is the portable path.
fn at_timestamp(datetime: &str) -> Result<String> {
    let dt = chrono::NaiveDateTime::parse_from_str(datetime, "%H:%M %Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Failed to parse datetime '{datetime}': {e}"))?;
    Ok(dt.format("%Y%m%d%H%M").to_string())
}

pub fn schedule_at_job(datetime: &str, command: &str) -> Result<u32> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("at")
        .arg("-t")
        .arg(at_timestamp(datetime)?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(command.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // at prints "job N at ..." to stderr
    for line in stderr.lines() {
        if line.contains("job") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "job"
                    && let Some(num_str) = parts.get(i + 1)
                    && let Ok(n) = num_str.parse::<u32>()
                {
                    return Ok(n);
                }
            }
        }
    }

    anyhow::bail!("Could not parse job number from at output: {}", stderr)
}

pub fn remove_at_job(job_number: u32) -> Result<()> {
    let status = Command::new("atrm")
        .arg(job_number.to_string())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("atrm failed for job {}", job_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_timestamp_conversion() {
        // internal "HH:MM YYYY-MM-DD" → POSIX `at -t` [[CC]YY]MMDDhhmm
        assert_eq!(at_timestamp("16:43 2026-08-01").unwrap(), "202608011643");
        assert_eq!(at_timestamp("05:02 2099-01-01").unwrap(), "209901010502");
        assert!(at_timestamp("now + 5 minutes").is_err());
    }

    #[test]
    fn test_parse_atq_output() {
        let atq_output = "42\tMon Mar 30 15:00:00 2026 a droshima\n43\tTue Mar 31 08:00:00 2026 a droshima\n";
        let entries = parse_atq_output(atq_output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].job_number, 42);
        assert_eq!(entries[1].job_number, 43);
    }

    #[test]
    fn test_parse_atq_empty() {
        let entries = parse_atq_output("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_at_available_detection() {
        let result = parse_at_availability_check(true);
        assert!(result);
        let result = parse_at_availability_check(false);
        assert!(!result);
    }

    #[test]
    fn test_parse_at_command_script() {
        let at_c_output = "#!/bin/sh\n# atrun uid=501 gid=20\nexport HOME=/Users/user\necho hello world\n";
        let command = extract_command_from_at_script(at_c_output);
        assert_eq!(command, "echo hello world");
    }
}
