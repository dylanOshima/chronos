use anyhow::Result;

/// Convert a cron expression to a human-readable description.
pub fn humanize_cron(cron_expr: &str) -> Result<String> {
    let description =
        cron_descriptor::cronparser::cron_expression_descriptor::get_description_cron(cron_expr)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to humanize cron expression '{cron_expr}': {}",
                    e.s
                )
            })?;
    Ok(description)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_day_at_8am() {
        let result = humanize_cron("0 8 * * *").unwrap();
        assert!(result.to_lowercase().contains("8"), "Expected '8' in: {result}");
    }

    #[test]
    fn test_every_sunday() {
        let result = humanize_cron("0 6 * * 0").unwrap();
        assert!(result.to_lowercase().contains("6"), "Expected '6' in: {result}");
    }

    #[test]
    fn test_every_30_minutes() {
        let result = humanize_cron("*/30 * * * *").unwrap();
        assert!(result.to_lowercase().contains("30"), "Expected '30' in: {result}");
    }
}
