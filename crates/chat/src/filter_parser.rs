use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use crate::types::{ChatId, ChatPattern};

/// Parse a source:pattern filter string
/// Examples:
/// - "telegram:Antti" -> source="telegram", pattern=Name("Antti")
/// - "telegram:123456" -> source="telegram", pattern=Id(ChatId("123456"))
/// - "telegram:*" -> source="telegram", pattern=All
/// - "*:*" -> source=None, pattern=All
pub fn parse_source_filter(input: &str) -> Result<(Option<String>, ChatPattern)> {
    let parts: Vec<&str> = input.splitn(2, ':').collect();

    match parts.as_slice() {
        [source, pattern] => {
            let source_id = if *source == "*" {
                None
            } else {
                Some(source.to_string())
            };

            let chat_pattern = parse_chat_pattern(pattern)?;

            Ok((source_id, chat_pattern))
        }
        [single] => {
            // No colon, treat as chat pattern in default source
            let chat_pattern = parse_chat_pattern(single)?;
            Ok((None, chat_pattern))
        }
        _ => {
            anyhow::bail!("Invalid filter format. Expected 'source:pattern'");
        }
    }
}

/// Parse a chat pattern
/// Examples:
/// - "*" -> All
/// - "123456" (numeric) -> Id(ChatId("123456"))
/// - "Antti" -> Name("Antti")
fn parse_chat_pattern(pattern: &str) -> Result<ChatPattern> {
    if pattern == "*" {
        Ok(ChatPattern::All)
    } else if pattern.chars().all(|c| c.is_ascii_digit() || c == '-') {
        // Numeric pattern is treated as ID
        Ok(ChatPattern::Id(ChatId::new(pattern)))
    } else {
        // Non-numeric is treated as name pattern
        Ok(ChatPattern::Name(pattern.to_string()))
    }
}

/// Parse a time specification into `DateTime<Utc>`
/// Supports:
/// - Relative: "7d", "2h", "30m", "60s"
/// - Absolute: "2025-01-15", "2025-01-15T14:30:00Z"
pub fn parse_time_spec(spec: &str) -> Result<DateTime<Utc>> {
    // Try parsing as relative time first
    if let Some(duration) = parse_relative_time(spec) {
        let now = Utc::now();
        return Ok(now - duration);
    }

    // Try parsing as absolute time
    // Try ISO datetime with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(spec) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try date only (assume midnight UTC)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(spec, "%Y-%m-%d") {
        let datetime = date.and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Ok(DateTime::from_naive_utc_and_offset(datetime, Utc));
    }

    anyhow::bail!("Invalid time specification: {}. Expected format: '7d', '2h', '2025-01-15', or ISO 8601 datetime", spec)
}

/// Parse relative time specification (7d, 2h, 30m, 60s)
fn parse_relative_time(spec: &str) -> Option<Duration> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    let (num_str, unit) = spec.split_at(spec.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    match unit {
        "s" => Some(Duration::seconds(num)),
        "m" => Some(Duration::minutes(num)),
        "h" => Some(Duration::hours(num)),
        "d" => Some(Duration::days(num)),
        "w" => Some(Duration::weeks(num)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_filter_with_name() {
        let (source, pattern) = parse_source_filter("telegram:Antti").unwrap();
        assert_eq!(source.as_deref(), Some("telegram"));
        assert_eq!(pattern, ChatPattern::Name("Antti".to_string()));
    }

    #[test]
    fn test_parse_source_filter_with_id() {
        let (source, pattern) = parse_source_filter("telegram:123456").unwrap();
        assert_eq!(source.as_deref(), Some("telegram"));
        assert_eq!(pattern, ChatPattern::Id(ChatId::new("123456")));
    }

    #[test]
    fn test_parse_source_filter_with_wildcard_pattern() {
        let (source, pattern) = parse_source_filter("telegram:*").unwrap();
        assert_eq!(source.as_deref(), Some("telegram"));
        assert_eq!(pattern, ChatPattern::All);
    }

    #[test]
    fn test_parse_source_filter_with_wildcard_source() {
        let (source, pattern) = parse_source_filter("*:*").unwrap();
        assert_eq!(source, None);
        assert_eq!(pattern, ChatPattern::All);
    }

    #[test]
    fn test_parse_source_filter_no_colon() {
        let (source, pattern) = parse_source_filter("Antti").unwrap();
        assert_eq!(source, None);
        assert_eq!(pattern, ChatPattern::Name("Antti".to_string()));
    }

    #[test]
    fn test_parse_relative_time_days() {
        let duration = parse_relative_time("7d").unwrap();
        assert_eq!(duration, Duration::days(7));
    }

    #[test]
    fn test_parse_relative_time_hours() {
        let duration = parse_relative_time("2h").unwrap();
        assert_eq!(duration, Duration::hours(2));
    }

    #[test]
    fn test_parse_relative_time_minutes() {
        let duration = parse_relative_time("30m").unwrap();
        assert_eq!(duration, Duration::minutes(30));
    }

    #[test]
    fn test_parse_relative_time_seconds() {
        let duration = parse_relative_time("60s").unwrap();
        assert_eq!(duration, Duration::seconds(60));
    }

    #[test]
    fn test_parse_relative_time_weeks() {
        let duration = parse_relative_time("2w").unwrap();
        assert_eq!(duration, Duration::weeks(2));
    }

    #[test]
    fn test_parse_time_spec_relative() {
        let dt = parse_time_spec("7d").unwrap();
        let expected = Utc::now() - Duration::days(7);
        // Allow 1 second difference for test execution time
        assert!((dt - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_time_spec_date_only() {
        let dt = parse_time_spec("2025-01-15").unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2025-01-15");
        assert_eq!(dt.format("%H:%M:%S").to_string(), "00:00:00");
    }

    #[test]
    fn test_parse_time_spec_iso_datetime() {
        let dt = parse_time_spec("2025-01-15T14:30:00Z").unwrap();
        assert_eq!(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(), "2025-01-15T14:30:00Z");
    }

    #[test]
    fn test_parse_time_spec_invalid() {
        let result = parse_time_spec("invalid");
        assert!(result.is_err());
    }
}
