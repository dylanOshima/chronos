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
    Command::new("which")
        .arg("at")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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

pub fn schedule_at_job(datetime: &str, command: &str) -> Result<u32> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("at")
        .arg(datetime)
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
