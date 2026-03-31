use anyhow::Result;
use crate::sidecar::{self, Sidecar};
use crate::system::{crontab, at};

pub fn run(identifier: &str) -> Result<()> {
    let sidecar_path = sidecar::sidecar_path();
    let mut sidecar = Sidecar::load(&sidecar_path)?;

    // Try to resolve identifier as a row number first
    if let Ok(row_num) = identifier.parse::<usize>() {
        return remove_by_index(row_num, &mut sidecar, &sidecar_path);
    }

    // Try to find by sidecar id (cron)
    if let Some(raw_line) = sidecar.find_cron_by_id(identifier) {
        let current = crontab::read_system_crontab()?;
        let new_content = crontab::remove_crontab_entry(&current, &raw_line);
        crontab::write_system_crontab(&new_content)?;
        sidecar.cron.remove(&raw_line);
        sidecar.save(&sidecar_path)?;
        println!("Removed cron job: {identifier}");
        return Ok(());
    }

    // Try to find by sidecar id (at)
    if let Some(job_number) = sidecar.find_at_by_id(identifier) {
        at::remove_at_job(job_number)?;
        sidecar.at.remove(&job_number.to_string());
        sidecar.save(&sidecar_path)?;
        println!("Removed at job: {identifier}");
        return Ok(());
    }

    anyhow::bail!("No job found with identifier '{identifier}'")
}

fn remove_by_index(row_num: usize, sidecar: &mut Sidecar, sidecar_path: &std::path::Path) -> Result<()> {
    let crontab_content = crontab::read_system_crontab()?;
    let cron_entries = crontab::parse_crontab(&crontab_content);
    let at_entries = at::read_at_queue().unwrap_or_default();

    let total_cron = cron_entries.len();

    if row_num == 0 {
        anyhow::bail!("Row numbers start at 1");
    }

    if row_num <= total_cron {
        let entry = &cron_entries[row_num - 1];
        let new_content = crontab::remove_crontab_entry(&crontab_content, &entry.raw_line);
        crontab::write_system_crontab(&new_content)?;
        sidecar.cron.remove(&entry.raw_line);
        sidecar.save(sidecar_path)?;
        println!("Removed cron job #{row_num}: {}", entry.command);
    } else if row_num <= total_cron + at_entries.len() {
        let at_index = row_num - total_cron - 1;
        let entry = &at_entries[at_index];
        at::remove_at_job(entry.job_number)?;
        sidecar.at.remove(&entry.job_number.to_string());
        sidecar.save(sidecar_path)?;
        println!("Removed at job #{row_num}");
    } else {
        anyhow::bail!("Row number {row_num} out of range (total jobs: {})", total_cron + at_entries.len());
    }

    Ok(())
}
