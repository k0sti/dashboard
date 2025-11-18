use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::OutputFormat;
use super::parse_time;
use crate::formatters;

pub async fn execute(
    chat: String,
    format: OutputFormat,
    output: String,
    since: Option<String>,
    before: Option<String>,
    limit: Option<usize>,
) -> Result<()> {
    println!("{}", format!("Exporting messages from '{}'...", chat).bold());

    // Parse time filters
    let since_time = if let Some(s) = since {
        Some(parse_time(&s)?)
    } else {
        None
    };

    let before_time = if let Some(b) = before {
        Some(parse_time(&b)?)
    } else {
        None
    };

    if let Some(st) = since_time {
        println!("  {}: {}", "Since".dimmed(), st.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(bt) = before_time {
        println!("  {}: {}", "Before".dimmed(), bt.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(l) = limit {
        println!("  {}: {}", "Limit".dimmed(), l);
    }

    // TODO: Implement with actual Telegram client
    let messages = vec![];

    let formatted = formatters::format_messages(&messages, format)?;
    std::fs::write(&output, &formatted)
        .context("Failed to write output file")?;

    println!();
    println!("{}", format!("Exported {} messages to: {}", messages.len(), output).green());

    Ok(())
}
