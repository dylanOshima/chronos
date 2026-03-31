use anyhow::Result;
use crate::sidecar::{self, Sidecar};
use crate::system::{crontab, at};

pub fn run_enable(identifier: &str) -> Result<()> {
    let raw_line = resolve_identifier(identifier)?;
    let current = crontab::read_system_crontab()?;
    let new_content = crontab::enable_crontab_entry(&current, &raw_line);
    crontab::write_system_crontab(&new_content)?;
    println!("Enabled job: {identifier}");
    Ok(())
}

pub fn run_disable(identifier: &str) -> Result<()> {
    let raw_line = resolve_identifier(identifier)?;
    let current = crontab::read_system_crontab()?;
    let new_content = crontab::disable_crontab_entry(&current, &raw_line);
    crontab::write_system_crontab(&new_content)?;
    println!("Disabled job: {identifier}");
    Ok(())
}

fn resolve_identifier(identifier: &str) -> Result<String> {
    if let Ok(row_num) = identifier.parse::<usize>() {
        let crontab_content = crontab::read_system_crontab()?;
        let entries = crontab::parse_crontab(&crontab_content);
        if row_num == 0 || row_num > entries.len() {
            // Check if this row number corresponds to an at job in the combined list
            let at_entries = at::read_at_queue().unwrap_or_default();
            let total_cron = entries.len();
            let total_combined = total_cron + at_entries.len();
            if row_num > total_cron && row_num <= total_combined {
                anyhow::bail!("Job #{row_num} is an at job — enable/disable only applies to cron jobs.");
            }
            anyhow::bail!("Row number {row_num} out of range (total cron jobs: {})", entries.len());
        }
        return Ok(entries[row_num - 1].raw_line.clone());
    }

    let sidecar_path = sidecar::sidecar_path();
    let sidecar = Sidecar::load(&sidecar_path)?;
    if let Some(raw_line) = sidecar.find_cron_by_id(identifier) {
        return Ok(raw_line);
    }

    anyhow::bail!("No cron job found with identifier '{identifier}'")
}
