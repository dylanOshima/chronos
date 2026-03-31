use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum JobType {
    Cron,
    At,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub index: usize,
    pub id: Option<String>,
    pub schedule_human: String,
    pub command: String,
    pub source: String,
    pub job_type: JobType,
    #[serde(skip)]
    pub raw_schedule: String,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_display_fields() {
        let job = Job {
            index: 1,
            id: Some("daily-brief".to_string()),
            schedule_human: "Every day at 8:00am".to_string(),
            command: "echo hello".to_string(),
            source: "claude".to_string(),
            job_type: JobType::Cron,
            raw_schedule: "0 8 * * *".to_string(),
            enabled: true,
        };
        assert_eq!(job.id, Some("daily-brief".to_string()));
        assert_eq!(job.source, "claude");
        assert!(job.enabled);
    }

    #[test]
    fn test_job_unknown_source() {
        let job = Job {
            index: 1,
            id: None,
            schedule_human: "Every Sunday at 2am".to_string(),
            command: "/usr/local/bin/backup.sh".to_string(),
            source: "unknown".to_string(),
            job_type: JobType::Cron,
            raw_schedule: "0 2 * * 0".to_string(),
            enabled: true,
        };
        assert!(job.id.is_none());
        assert_eq!(job.source, "unknown");
    }
}
