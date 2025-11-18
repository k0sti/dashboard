use anyhow::Result;
use colored::Colorize;

use crate::cli::OutputFormat;
use crate::formatters;

pub async fn execute(
    chat: Option<String>,
    term: String,
    all: bool,
    ignore_case: bool,
    format: OutputFormat,
) -> Result<()> {
    if all {
        println!("{}", format!("Searching all chats for '{}'...", term).bold());
    } else if let Some(ref chat_name) = chat {
        println!("{}", format!("Searching '{}' for '{}'...", chat_name, term).bold());
    } else {
        anyhow::bail!("Either provide a chat name or use --all flag");
    }

    if ignore_case {
        println!("  {}: enabled", "Case-insensitive".dimmed());
    }

    // TODO: Implement with actual Telegram client
    let messages = vec![];

    if messages.is_empty() {
        println!();
        println!("{}", "No messages found matching the search term.".yellow());
        return Ok(());
    }

    let formatted = formatters::format_messages(&messages, format)?;
    println!("{}", formatted);

    Ok(())
}
