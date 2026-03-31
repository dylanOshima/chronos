use anyhow::Result;
use crate::schedule::{classify_schedule, ScheduleKind};
use crate::sidecar::{self, JobMeta, Sidecar};
use crate::system::{crontab, at};

pub fn run(
    schedule: String,
    command: String,
    id: Option<String>,
    desc: Option<String>,
    source: Option<String>,
    force: bool,
) -> Result<()> {
    let kind = classify_schedule(&schedule)?;
    let source = source.unwrap_or_else(|| {
        std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    });

    match kind {
        ScheduleKind::Recurring { cron_expr } => {
            add_recurring(&cron_expr, &command, id, desc, &source, force)
        }
        ScheduleKind::OneOff { at_time } => {
            add_oneoff(&at_time, &command, id, desc, &source)
        }
    }
}

fn add_recurring(
    cron_expr: &str,
    command: &str,
    id: Option<String>,
    desc: Option<String>,
    source: &str,
    force: bool,
) -> Result<()> {
    let current = crontab::read_system_crontab()?;
    let raw_line = format!("{cron_expr} {command}");

    // Duplicate check
    if !force {
        let entries = crontab::parse_crontab(&current);
        if entries.iter().any(|e| e.raw_line == raw_line) {
            anyhow::bail!(
                "Duplicate job detected: '{raw_line}' already exists. Use --force to add anyway."
            );
        }
    }

    let new_content = crontab::add_crontab_entry(&current, cron_expr, command);
    crontab::write_system_crontab(&new_content)?;

    // Update sidecar
    let sidecar_path = sidecar::sidecar_path();
    let mut sidecar = Sidecar::load(&sidecar_path)?;
    sidecar.set_cron_meta(
        &raw_line,
        JobMeta {
            id: id.clone(),
            description: desc,
            source: Some(source.to_string()),
        },
    );
    sidecar.save(&sidecar_path)?;

    let display_id = id.as_deref().unwrap_or(&raw_line);
    println!("Added recurring job: {display_id}");
    Ok(())
}

fn add_oneoff(
    at_time: &str,
    command: &str,
    id: Option<String>,
    desc: Option<String>,
    source: &str,
) -> Result<()> {
    if at::is_at_available() {
        let job_number = at::schedule_at_job(at_time, command)?;

        let sidecar_path = sidecar::sidecar_path();
        let mut sidecar = Sidecar::load(&sidecar_path)?;
        sidecar.set_at_meta(
            job_number,
            JobMeta {
                id: id.clone(),
                description: desc,
                source: Some(source.to_string()),
            },
        );
        sidecar.save(&sidecar_path)?;

        let job_number_str = job_number.to_string();
        let display_id = id.as_deref().unwrap_or(&job_number_str);
        println!("Added one-off job: {display_id} (at job #{job_number})");
    } else {
        add_oneoff_via_cron_fallback(at_time, command, id, desc, source)?;
    }
    Ok(())
}

fn add_oneoff_via_cron_fallback(
    at_time: &str,
    command: &str,
    id: Option<String>,
    desc: Option<String>,
    source: &str,
) -> Result<()> {
    let dt = chrono::NaiveDateTime::parse_from_str(at_time, "%H:%M %Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Failed to parse datetime '{at_time}': {e}"))?;

    let minute = dt.format("%M").to_string();
    let hour = dt.format("%H").to_string();
    let day = dt.format("%-d").to_string();
    let month = dt.format("%-m").to_string();

    let job_id = id.clone().unwrap_or_else(|| format!("oneoff-{}", dt.and_utc().timestamp()));
    let cleanup_id = format!("{job_id}-cleanup");

    let chronos_bin = std::env::current_exe()?.to_string_lossy().to_string();

    let job_cron = format!("{minute} {hour} {day} {month} *");
    let job_command = format!(
        "{chronos_bin} _run-once {job_id} --cleanup-id {cleanup_id} -- {command}"
    );

    let cleanup_minute: u32 = minute.parse::<u32>().unwrap_or(0) + 1;
    let cleanup_cron = if cleanup_minute >= 60 {
        let cleanup_hour: u32 = hour.parse::<u32>().unwrap_or(0) + 1;
        format!("0 {cleanup_hour} {day} {month} *")
    } else {
        format!("{cleanup_minute} {hour} {day} {month} *")
    };
    let cleanup_command = format!(
        "{chronos_bin} remove {job_id} && {chronos_bin} remove {cleanup_id}"
    );

    let current = crontab::read_system_crontab()?;
    let with_job = crontab::add_crontab_entry(&current, &job_cron, &job_command);
    let with_cleanup = crontab::add_crontab_entry(&with_job, &cleanup_cron, &cleanup_command);
    crontab::write_system_crontab(&with_cleanup)?;

    let sidecar_path = sidecar::sidecar_path();
    let mut sidecar = Sidecar::load(&sidecar_path)?;
    sidecar.set_cron_meta(
        &format!("{job_cron} {job_command}"),
        JobMeta {
            id: Some(job_id.clone()),
            description: desc,
            source: Some(source.to_string()),
        },
    );
    sidecar.set_cron_meta(
        &format!("{cleanup_cron} {cleanup_command}"),
        JobMeta {
            id: Some(cleanup_id.clone()),
            description: Some(format!("Cleanup for {job_id}")),
            source: Some("chronos".to_string()),
        },
    );
    sidecar.save(&sidecar_path)?;

    println!("Added one-off job: {job_id} (via self-destructing cron, at not available)");
    Ok(())
}
