use anyhow::Result;
use super::cron_gen::natural_to_cron;

#[derive(Debug, Clone)]
pub enum ScheduleKind {
    /// A recurring schedule with a cron expression.
    Recurring { cron_expr: String },
    /// A one-off schedule with an absolute datetime string (for `at`).
    OneOff { at_time: String },
}

const RECURRING_PREFIXES: &[&str] = &["every ", "daily ", "weekly ", "monthly "];
const RECURRING_EXACT: &[&str] = &["daily", "weekly", "monthly", "hourly"];

/// Classify a schedule string as recurring or one-off.
/// For now, stores the original input in the variant — Tasks 8/9 add real conversion.
pub fn classify_schedule(input: &str) -> Result<ScheduleKind> {
    let trimmed = input.trim().to_lowercase();

    // Check if it's a raw cron expression (5 space-separated fields)
    if is_cron_expression(&trimmed) {
        return Ok(ScheduleKind::Recurring {
            cron_expr: trimmed.to_string(),
        });
    }

    // Check for recurring keywords
    if RECURRING_EXACT.contains(&trimmed.as_str()) {
        let cron_expr = natural_to_cron(&trimmed)?;
        return Ok(ScheduleKind::Recurring { cron_expr });
    }
    for prefix in RECURRING_PREFIXES {
        if trimmed.starts_with(prefix) {
            let cron_expr = natural_to_cron(&trimmed)?;
            return Ok(ScheduleKind::Recurring { cron_expr });
        }
    }

    // Otherwise, treat as a one-off
    Ok(ScheduleKind::OneOff {
        at_time: trimmed.to_string(),
    })
}

fn is_cron_expression(input: &str) -> bool {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    parts.iter().all(|p| {
        p.chars().all(|c| c.is_ascii_digit() || "*/,-".contains(c))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_expression_detected_as_recurring() {
        let result = classify_schedule("0 8 * * *").unwrap();
        assert!(matches!(result, ScheduleKind::Recurring { .. }));
    }

    #[test]
    fn test_every_keyword_is_recurring() {
        let result = classify_schedule("every sunday at 6am").unwrap();
        assert!(matches!(result, ScheduleKind::Recurring { .. }));
    }

    #[test]
    fn test_daily_keyword_is_recurring() {
        let result = classify_schedule("daily at midnight").unwrap();
        assert!(matches!(result, ScheduleKind::Recurring { .. }));
    }

    #[test]
    fn test_weekday_keyword_is_recurring() {
        let result = classify_schedule("every weekday at 9am").unwrap();
        assert!(matches!(result, ScheduleKind::Recurring { .. }));
    }

    #[test]
    fn test_tomorrow_is_oneoff() {
        let result = classify_schedule("tomorrow at 1am").unwrap();
        assert!(matches!(result, ScheduleKind::OneOff { .. }));
    }

    #[test]
    fn test_specific_date_is_oneoff() {
        let result = classify_schedule("march 31 at noon").unwrap();
        assert!(matches!(result, ScheduleKind::OneOff { .. }));
    }

    #[test]
    fn test_bare_day_name_is_oneoff() {
        let result = classify_schedule("sunday 6pm").unwrap();
        assert!(matches!(result, ScheduleKind::OneOff { .. }));
    }
}
