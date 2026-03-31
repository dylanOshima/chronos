use anyhow::Result;
use crate::job::{Job, JobType};
use crate::output;
use crate::sidecar::{self, Sidecar};
use crate::system::crontab;
use crate::system::at;
use crate::schedule::humanize::humanize_cron;

pub fn run(json: bool) -> Result<()> {
    let sidecar_path = sidecar::sidecar_path();
    let mut sidecar = Sidecar::load(&sidecar_path)?;
    let mut jobs = Vec::new();
    let mut index = 1;

    // Read crontab entries
    let crontab_content = crontab::read_system_crontab()?;
    let cron_entries = crontab::parse_crontab(&crontab_content);

    for entry in &cron_entries {
        let meta = sidecar.get_cron_meta(&entry.raw_line);
        let schedule_human = humanize_cron(&entry.schedule).unwrap_or_else(|_| entry.schedule.clone());
        jobs.push(Job {
            index,
            id: meta.and_then(|m| m.id.clone()),
            schedule_human,
            command: entry.command.clone(),
            source: meta
                .and_then(|m| m.source.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            job_type: JobType::Cron,
            raw_schedule: entry.schedule.clone(),
            enabled: entry.enabled,
        });
        index += 1;
    }

    // Read at queue entries
    let at_entries = at::read_at_queue().unwrap_or_default();
    for entry in &at_entries {
        let meta = sidecar.get_at_meta(entry.job_number);
        jobs.push(Job {
            index,
            id: meta.and_then(|m| m.id.clone()),
            schedule_human: entry.scheduled_time.clone(),
            command: entry.command.clone().unwrap_or_else(|| "\u{2014}".to_string()),
            source: meta
                .and_then(|m| m.source.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            job_type: JobType::At,
            raw_schedule: String::new(),
            enabled: true,
        });
        index += 1;
    }

    // Prune stale sidecar entries
    let active_cron_lines: Vec<String> = cron_entries.iter().map(|e| e.raw_line.clone()).collect();
    let active_at_jobs: Vec<u32> = at_entries.iter().map(|e| e.job_number).collect();
    sidecar.prune(&active_cron_lines, &active_at_jobs);
    let _ = sidecar.save(&sidecar_path);

    // Output
    if json {
        println!("{}", output::format_json(&jobs)?);
    } else {
        println!("{}", output::format_table(&jobs));
    }

    Ok(())
}
