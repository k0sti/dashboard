use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::OutputFormat;
use super::parse_time;
use crate::formatters;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    chat: Option<String>,
    id: Option<String>,
    limit: usize,
    since: Option<String>,
    before: Option<String>,
    after: Option<String>,
    sender: Option<String>,
    message_type: Option<String>,
    format: OutputFormat,
    output: Option<String>,
) -> Result<()> {
    let chat_id = chat.or(id).context("Chat name or ID is required")?;

    println!("{}", format!("Fetching messages from '{}'...", chat_id).dimmed());

    // Parse time filters if provided
    let since_time = if let Some(s) = since.or(after) {
        Some(parse_time(&s)?)
    } else {
        None
    };

    let before_time = if let Some(b) = before {
        Some(parse_time(&b)?)
    } else {
        None
    };

    // Log filter info
    if let Some(st) = since_time {
        println!("  {}: {}", "Since".dimmed(), st.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(bt) = before_time {
        println!("  {}: {}", "Before".dimmed(), bt.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(ref s) = sender {
        println!("  {}: {}", "Sender filter".dimmed(), s);
    }
    if let Some(ref t) = message_type {
        println!("  {}: {}", "Type filter".dimmed(), t);
    }
    println!("  {}: {}", "Limit".dimmed(), limit);

    // TODO: Implement with actual Telegram client
    let messages = vec![];

    if messages.is_empty() {
        println!();
        println!("{}", "No messages found.".yellow());
        println!();
        println!("{}", "Note:".yellow().bold());
        println!("  The Telegram client implementation is not yet complete.");
        println!("  To retrieve messages, implement TelegramChatClient in the chat crate");
        println!("  and integrate it here.");
        return Ok(());
    }

    let formatted = formatters::format_messages(&messages, format)?;

    if let Some(output_file) = output {
        std::fs::write(&output_file, &formatted)
            .context("Failed to write output file")?;
        println!("{}", format!("Output written to: {}", output_file).green());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}
