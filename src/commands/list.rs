use anyhow::Result;
use crate::output;
use crate::sidecar;
use crate::system::{crontab, at};
use super::common::gather_all_jobs;

pub fn run(json: bool) -> Result<()> {
    let sidecar_path = sidecar::sidecar_path();
    let (jobs, mut sidecar) = gather_all_jobs()?;

    // Prune stale sidecar entries — re-read entries for active keys
    let crontab_content = crontab::read_system_crontab()?;
    let cron_entries = crontab::parse_crontab(&crontab_content);
    let at_entries = at::read_at_queue().unwrap_or_default();

    let active_cron_lines: Vec<String> = cron_entries.iter().map(|e| e.raw_line.clone()).collect();
    let active_at_jobs: Vec<u32> = at_entries.iter().map(|e| e.job_number).collect();
    sidecar.prune(&active_cron_lines, &active_at_jobs);
    let _ = sidecar.save(&sidecar_path);

    if json {
        println!("{}", output::format_json(&jobs)?);
    } else {
        println!("{}", output::format_table(&jobs));
    }

    Ok(())
}
