use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattedDuration(pub Duration);

impl fmt::Display for FormattedDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_secs = self.0.as_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        if hours > 0 {
            write!(f, "{}h{:02}m{:02}s", hours, mins, secs)
        } else if mins > 0 {
            write!(f, "{}m{:02}s", mins, secs)
        } else {
            write!(f, "{}s", secs)
        }
    }
}

pub fn format_duration(d: Duration) -> String {
    FormattedDuration(d).to_string()
}

pub fn format_timer_clock(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("empty duration".to_string());
    }

    // Try purely numeric (treat as seconds)
    if let Ok(secs) = trimmed.parse::<u64>() {
        return Ok(Duration::from_secs(secs));
    }

    let mut total_millis: u64 = 0;
    let mut num_buf = String::new();
    let mut unit_buf = String::new();
    let mut parsed_any = false;

    let mut chars = trimmed.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_whitespace() {
            chars.next();
            continue;
        }

        if c.is_ascii_digit() {
            if !unit_buf.is_empty() {
                let millis = parse_unit_val(&num_buf, &unit_buf)?;
                total_millis = total_millis
                    .checked_add(millis)
                    .ok_or_else(|| "duration overflow".to_string())?;
                num_buf.clear();
                unit_buf.clear();
            }
            num_buf.push(c);
            chars.next();
            parsed_any = true;
        } else if c.is_ascii_alphabetic() {
            if num_buf.is_empty() {
                return Err(format!("invalid duration `{}`: unit without number", s));
            }
            unit_buf.push(c);
            chars.next();
        } else {
            return Err(format!(
                "invalid duration `{}`: unexpected character `{}`",
                s, c
            ));
        }
    }

    if !num_buf.is_empty() {
        if unit_buf.is_empty() {
            return Err(format!("invalid duration `{}`: missing unit at end", s));
        }
        let millis = parse_unit_val(&num_buf, &unit_buf)?;
        total_millis = total_millis
            .checked_add(millis)
            .ok_or_else(|| "duration overflow".to_string())?;
    } else if !unit_buf.is_empty() {
        return Err(format!("invalid duration `{}`: dangling unit", s));
    }

    if !parsed_any {
        return Err(format!("invalid duration `{}`", s));
    }

    Ok(Duration::from_millis(total_millis))
}

fn parse_unit_val(num_str: &str, unit_str: &str) -> Result<u64, String> {
    let val: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number in duration `{}`", num_str))?;

    match unit_str.to_ascii_lowercase().as_str() {
        "ms" | "millis" | "milliseconds" => Ok(val),
        "s" | "sec" | "secs" | "second" | "seconds" => val
            .checked_mul(1000)
            .ok_or_else(|| "duration overflow".to_string()),
        "m" | "min" | "mins" | "minute" | "minutes" => val
            .checked_mul(60 * 1000)
            .ok_or_else(|| "duration overflow".to_string()),
        "h" | "hr" | "hrs" | "hour" | "hours" => val
            .checked_mul(3600 * 1000)
            .ok_or_else(|| "duration overflow".to_string()),
        _ => Err(format!("unknown duration unit `{}`", unit_str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(1)), "1s");
        assert_eq!(format_duration(Duration::from_secs(7)), "7s");
        assert_eq!(format_duration(Duration::from_secs(42)), "42s");
        assert_eq!(format_duration(Duration::from_secs(63)), "1m03s");
        assert_eq!(format_duration(Duration::from_secs(134)), "2m14s");
        assert_eq!(format_duration(Duration::from_secs(3729)), "1h02m09s");
    }

    #[test]
    fn test_format_timer_clock() {
        assert_eq!(format_timer_clock(Duration::from_secs(0)), "0:00");
        assert_eq!(format_timer_clock(Duration::from_secs(42)), "0:42");
        assert_eq!(format_timer_clock(Duration::from_secs(134)), "2:14");
        assert_eq!(format_timer_clock(Duration::from_secs(3729)), "1:02:09");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("2m30s").unwrap(), Duration::from_secs(150));
        assert_eq!(parse_duration("1h2m3s").unwrap(), Duration::from_secs(3723));
        assert_eq!(parse_duration("42").unwrap(), Duration::from_secs(42));
        assert!(parse_duration("banana").is_err());
        assert!(parse_duration("").is_err());
    }
}
