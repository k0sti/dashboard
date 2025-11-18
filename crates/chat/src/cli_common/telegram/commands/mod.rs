pub mod client;
pub mod config_cmd;
pub mod export;
pub mod get;
pub mod info;
pub mod init;
pub mod list;
pub mod logout;
pub mod search;
pub mod status;
pub mod watch;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};

/// Parse time string (absolute or relative)
pub fn parse_time(time_str: &str) -> Result<DateTime<Utc>> {
    // Try parsing as RFC3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(time_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try parsing as relative time using humantime
    if let Ok(duration) = humantime::parse_duration(time_str) {
        let chrono_duration = Duration::from_std(duration)
            .context("Duration too large")?;
        return Ok(Utc::now() - chrono_duration);
    }

    anyhow::bail!("Invalid time format. Use RFC3339 (e.g., '2025-01-01T00:00:00Z') or relative time (e.g., '2 days ago')")
}
