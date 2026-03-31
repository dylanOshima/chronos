use anyhow::{bail, Result};

/// Convert a natural language recurring schedule to a cron expression.
pub fn natural_to_cron(input: &str) -> Result<String> {
    let s = input.trim().to_lowercase();

    // Bare keywords
    match s.as_str() {
        "hourly" => return Ok("0 * * * *".to_string()),
        "daily" => return Ok("0 0 * * *".to_string()),
        "weekly" => return Ok("0 0 * * 0".to_string()),
        "monthly" => return Ok("0 0 1 * *".to_string()),
        _ => {}
    }

    // "every N minutes"
    if let Some(rest) = s.strip_prefix("every ") {
        if let Some(n_str) = rest.strip_suffix(" minutes") {
            let n: u32 = n_str
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid minute interval: {}", n_str))?;
            return Ok(format!("*/{} * * * *", n));
        }

        // "every N hours"
        if let Some(n_str) = rest.strip_suffix(" hours") {
            let n: u32 = n_str
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid hour interval: {}", n_str))?;
            return Ok(format!("0 */{} * * *", n));
        }

        // "every weekday at TIME"
        if let Some(time_str) = rest.strip_prefix("weekday at ") {
            let (h, m) = parse_time(time_str)
                .ok_or_else(|| anyhow::anyhow!("cannot parse time: {}", time_str))?;
            return Ok(format!("{} {} * * 1-5", m, h));
        }

        // "every day at TIME"
        if let Some(time_str) = rest.strip_prefix("day at ") {
            let (h, m) = parse_time(time_str)
                .ok_or_else(|| anyhow::anyhow!("cannot parse time: {}", time_str))?;
            return Ok(format!("{} {} * * *", m, h));
        }

        // "every <dayname> at TIME"
        if let Some(at_pos) = rest.find(" at ") {
            let day_str = &rest[..at_pos];
            let time_str = &rest[at_pos + 4..];
            if let Some(day_num) = day_to_cron(day_str) {
                let (h, m) = parse_time(time_str)
                    .ok_or_else(|| anyhow::anyhow!("cannot parse time: {}", time_str))?;
                return Ok(format!("{} {} * * {}", m, h, day_num));
            }
        }
    }

    // "daily at TIME"
    if let Some(time_str) = s.strip_prefix("daily at ") {
        let (h, m) = parse_time(time_str)
            .ok_or_else(|| anyhow::anyhow!("cannot parse time: {}", time_str))?;
        return Ok(format!("{} {} * * *", m, h));
    }

    bail!("unrecognised recurring schedule: {}", input)
}

/// Map a day name to its cron day-of-week number (sunday=0 … saturday=6).
pub fn day_to_cron(day: &str) -> Option<&'static str> {
    match day.trim() {
        "sunday" => Some("0"),
        "monday" => Some("1"),
        "tuesday" => Some("2"),
        "wednesday" => Some("3"),
        "thursday" => Some("4"),
        "friday" => Some("5"),
        "saturday" => Some("6"),
        _ => None,
    }
}

/// Parse a time expression into (hour, minute).
/// Handles: "8am", "8pm", "8:30am", "8:30pm", "midnight", "noon", "13:00".
pub fn parse_time(input: &str) -> Option<(u32, u32)> {
    let s = input.trim();

    if s == "midnight" {
        return Some((0, 0));
    }
    if s == "noon" {
        return Some((12, 0));
    }

    // Suffixed: ends with "am" or "pm"
    if let Some(rest) = s.strip_suffix("am") {
        let (h, m) = parse_hm(rest)?;
        let h = if h == 12 { 0 } else { h };
        return Some((h, m));
    }
    if let Some(rest) = s.strip_suffix("pm") {
        let (h, m) = parse_hm(rest)?;
        let h = if h == 12 { 12 } else { h + 12 };
        return Some((h, m));
    }

    // Bare 24-hour "HH:MM"
    let (h, m) = parse_hm(s)?;
    Some((h, m))
}

/// Parse "H", "H:M", or "HH:MM" into (hour, minute).
fn parse_hm(s: &str) -> Option<(u32, u32)> {
    if let Some((h_str, m_str)) = s.split_once(':') {
        let h: u32 = h_str.parse().ok()?;
        let m: u32 = m_str.parse().ok()?;
        Some((h, m))
    } else {
        let h: u32 = s.parse().ok()?;
        Some((h, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_sunday_at_6am() {
        let result = natural_to_cron("every sunday at 6am").unwrap();
        assert_eq!(result, "0 6 * * 0");
    }

    #[test]
    fn test_daily_at_midnight() {
        let result = natural_to_cron("daily at midnight").unwrap();
        assert_eq!(result, "0 0 * * *");
    }

    #[test]
    fn test_every_weekday_at_9am() {
        let result = natural_to_cron("every weekday at 9am").unwrap();
        assert_eq!(result, "0 9 * * 1-5");
    }

    #[test]
    fn test_every_2_hours() {
        let result = natural_to_cron("every 2 hours").unwrap();
        assert_eq!(result, "0 */2 * * *");
    }

    #[test]
    fn test_every_30_minutes() {
        let result = natural_to_cron("every 30 minutes").unwrap();
        assert_eq!(result, "*/30 * * * *");
    }

    #[test]
    fn test_every_monday_at_8_30am() {
        let result = natural_to_cron("every monday at 8:30am").unwrap();
        assert_eq!(result, "30 8 * * 1");
    }

    #[test]
    fn test_daily_bare() {
        let result = natural_to_cron("daily").unwrap();
        assert_eq!(result, "0 0 * * *");
    }

    #[test]
    fn test_hourly() {
        let result = natural_to_cron("hourly").unwrap();
        assert_eq!(result, "0 * * * *");
    }

    #[test]
    fn test_weekly() {
        let result = natural_to_cron("weekly").unwrap();
        assert_eq!(result, "0 0 * * 0");
    }

    #[test]
    fn test_monthly() {
        let result = natural_to_cron("monthly").unwrap();
        assert_eq!(result, "0 0 1 * *");
    }

    #[test]
    fn test_every_day_at_8am() {
        let result = natural_to_cron("every day at 8am").unwrap();
        assert_eq!(result, "0 8 * * *");
    }
}
