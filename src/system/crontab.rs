use anyhow::Result;
use std::process::{Command, Stdio};
use std::io::Write as IoWrite;

#[derive(Debug, Clone)]
pub struct CrontabEntry {
    pub schedule: String,
    pub command: String,
    pub enabled: bool,
    pub raw_line: String,
}

/// Try to parse a line as a 5-field cron schedule + command.
/// Returns Some((schedule, command)) if the line is a valid cron entry.
fn try_parse_cron_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(6, ' ').collect();
    if parts.len() < 6 {
        return None;
    }
    // Validate that first 5 fields look like cron fields (digits, *, /, -, ,)
    let valid_cron_char = |s: &str| {
        s.chars().all(|c| c.is_ascii_digit() || matches!(c, '*' | '/' | '-' | ','))
    };
    for field in &parts[..5] {
        if !valid_cron_char(field) {
            return None;
        }
    }
    let schedule = parts[..5].join(" ");
    let command = parts[5].to_string();
    Some((schedule, command))
}

/// Parse crontab text into entries.
/// Active entries are enabled; commented-out cron lines are disabled.
/// Pure comments and environment variable lines are skipped.
pub fn parse_crontab(content: &str) -> Vec<CrontabEntry> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            // Strip the leading # and any optional space, then check if it's a cron line
            let after_hash = trimmed[1..].trim_start();
            if let Some((schedule, command)) = try_parse_cron_line(after_hash) {
                entries.push(CrontabEntry {
                    schedule,
                    command,
                    enabled: false,
                    raw_line: after_hash.to_string(),
                });
            }
            // Otherwise it's a pure comment — skip
            continue;
        }

        // Check for environment variable assignment (KEY=value)
        if trimmed.contains('=') {
            let before_eq = trimmed.split('=').next().unwrap_or("").trim();
            if before_eq.chars().all(|c| c.is_alphanumeric() || c == '_') && !before_eq.is_empty() {
                // Looks like an env var line — skip
                continue;
            }
        }

        // Try to parse as an active cron entry
        if let Some((schedule, command)) = try_parse_cron_line(trimmed) {
            entries.push(CrontabEntry {
                schedule,
                command,
                enabled: true,
                raw_line: trimmed.to_string(),
            });
        }
    }

    entries
}

/// Append a new cron entry to the crontab content.
pub fn add_crontab_entry(content: &str, schedule: &str, command: &str) -> String {
    let new_line = format!("{} {}", schedule, command);
    if content.ends_with('\n') || content.is_empty() {
        format!("{}{}\n", content, new_line)
    } else {
        format!("{}\n{}\n", content, new_line)
    }
}

/// Remove an entry (by its raw_line) from crontab content.
/// Removes both the active form and the commented-out form.
pub fn remove_crontab_entry(content: &str, raw_line: &str) -> String {
    let commented = format!("#{}", raw_line);
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed != raw_line && trimmed != commented
        })
        .map(|line| format!("{}\n", line))
        .collect()
}

/// Comment out a cron entry (disable it).
pub fn disable_crontab_entry(content: &str, raw_line: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.trim() == raw_line {
                format!("#{}\n", line.trim())
            } else {
                format!("{}\n", line)
            }
        })
        .collect()
}

/// Uncomment a cron entry (enable it).
pub fn enable_crontab_entry(content: &str, raw_line: &str) -> String {
    let commented = format!("#{}", raw_line);
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed == commented {
                format!("{}\n", raw_line)
            } else {
                format!("{}\n", line)
            }
        })
        .collect()
}

/// Run `crontab -l` and return its output. Returns empty string if no crontab exists.
pub fn read_system_crontab() -> Result<String> {
    let output = Command::new("crontab").arg("-l").output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        // Exit code 1 with "no crontab for user" message means empty crontab
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no crontab for") {
            Ok(String::new())
        } else {
            Err(anyhow::anyhow!(
                "crontab -l failed: {}",
                stderr
            ))
        }
    }
}

/// Pipe content to `crontab -` to write the system crontab.
pub fn write_system_crontab(content: &str) -> Result<()> {
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("crontab - failed with status: {}", status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CRONTAB: &str = "\
# some user comment
0 8 * * * echo hello
# disabled by chronos
#0 9 * * * echo disabled
30 2 * * 0 /usr/local/bin/backup.sh
";

    #[test]
    fn test_parse_crontab_entries() {
        let entries = parse_crontab(SAMPLE_CRONTAB);
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_parse_active_entry() {
        let entries = parse_crontab(SAMPLE_CRONTAB);
        let active = &entries[0];
        assert_eq!(active.schedule, "0 8 * * *");
        assert_eq!(active.command, "echo hello");
        assert!(active.enabled);
    }

    #[test]
    fn test_parse_disabled_entry() {
        let entries = parse_crontab(SAMPLE_CRONTAB);
        let disabled = &entries[1];
        assert_eq!(disabled.schedule, "0 9 * * *");
        assert_eq!(disabled.command, "echo disabled");
        assert!(!disabled.enabled);
    }

    #[test]
    fn test_add_entry_to_crontab() {
        let original = "0 8 * * * echo hello\n";
        let result = add_crontab_entry(original, "0 6 * * 0", "echo sunday");
        assert!(result.contains("0 8 * * * echo hello"));
        assert!(result.contains("0 6 * * 0 echo sunday"));
    }

    #[test]
    fn test_remove_entry_from_crontab() {
        let original = "0 8 * * * echo hello\n0 6 * * 0 echo sunday\n";
        let result = remove_crontab_entry(original, "0 8 * * * echo hello");
        assert!(!result.contains("echo hello"));
        assert!(result.contains("echo sunday"));
    }

    #[test]
    fn test_disable_entry() {
        let original = "0 8 * * * echo hello\n";
        let result = disable_crontab_entry(original, "0 8 * * * echo hello");
        assert!(result.contains("#0 8 * * * echo hello"));
    }

    #[test]
    fn test_enable_entry() {
        let original = "#0 8 * * * echo hello\n";
        let result = enable_crontab_entry(original, "0 8 * * * echo hello");
        assert!(result.contains("0 8 * * * echo hello"));
        assert!(!result.contains("#0 8 * * *"));
    }

    #[test]
    fn test_preserves_non_cron_lines() {
        let original = "# user comment\n0 8 * * * echo hello\nMAILTO=me@example.com\n";
        let result = remove_crontab_entry(original, "0 8 * * * echo hello");
        assert!(result.contains("# user comment"));
        assert!(result.contains("MAILTO=me@example.com"));
    }
}
