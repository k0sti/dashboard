use anyhow::Result;
use colored::Colorize;

use crate::cli::{ChatTypeFilter, OutputFormat};
use crate::formatters;

pub async fn execute(format: OutputFormat, chat_type: Option<ChatTypeFilter>) -> Result<()> {
    println!("{}", "Fetching chat list...".dimmed());

    // TODO: Implement with actual Telegram client
    // For now, return a stub message
    let chats = vec![];

    if chats.is_empty() {
        println!();
        println!("{}", "No chats found.".yellow());
        println!();
        println!("{}", "Note:".yellow().bold());
        println!("  The Telegram client implementation is not yet complete.");
        println!("  To list chats, implement TelegramChatClient in the chat crate");
        println!("  and integrate it here.");
        return Ok(());
    }

    let filtered_chats = if let Some(_filter) = chat_type {
        // TODO: Apply filter
        chats
    } else {
        chats
    };

    let output = formatters::format_chats(&filtered_chats, format)?;
    println!("{}", output);

    Ok(())
}
