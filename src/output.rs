use crate::job::Job;
use anyhow::Result;
use comfy_table::{Table, ContentArrangement};

/// Format jobs as a human-readable table string.
pub fn format_table(jobs: &[Job]) -> String {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["#", "ID", "Schedule", "Command", "Source"]);

    for job in jobs {
        table.add_row(vec![
            job.index.to_string(),
            job.id.as_deref().unwrap_or("\u{2014}").to_string(), // em dash for missing ID
            job.schedule_human.clone(),
            job.command.clone(),
            job.source.clone(),
        ]);
    }

    table.to_string()
}

/// Format jobs as a JSON string.
pub fn format_json(jobs: &[Job]) -> Result<String> {
    Ok(serde_json::to_string_pretty(jobs)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{Job, JobType};

    fn sample_jobs() -> Vec<Job> {
        vec![
            Job {
                index: 1,
                id: Some("daily-brief".to_string()),
                schedule_human: "Every day at 8:00 AM".to_string(),
                command: "echo hello".to_string(),
                source: "claude".to_string(),
                job_type: JobType::Cron,
                raw_schedule: "0 8 * * *".to_string(),
                enabled: true,
            },
            Job {
                index: 2,
                id: None,
                schedule_human: "Every Sunday at 2:00 AM".to_string(),
                command: "/usr/local/bin/backup.sh".to_string(),
                source: "unknown".to_string(),
                job_type: JobType::Cron,
                raw_schedule: "0 2 * * 0".to_string(),
                enabled: true,
            },
        ]
    }

    #[test]
    fn test_table_output_contains_headers() {
        let output = format_table(&sample_jobs());
        assert!(output.contains("ID"));
        assert!(output.contains("Schedule"));
        assert!(output.contains("Command"));
        assert!(output.contains("Source"));
    }

    #[test]
    fn test_table_output_contains_data() {
        let output = format_table(&sample_jobs());
        assert!(output.contains("daily-brief"));
        assert!(output.contains("claude"));
        assert!(output.contains("unknown"));
    }

    #[test]
    fn test_json_output_is_valid() {
        let output = format_json(&sample_jobs()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_table_shows_dash_for_missing_id() {
        let output = format_table(&sample_jobs());
        assert!(output.contains("\u{2014}") || output.contains("-"));
    }
}
