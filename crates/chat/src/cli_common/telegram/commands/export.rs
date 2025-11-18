use anyhow::{Context, Result};
use chat::{ChatId, Message, MessageContent, MessageId, User, UserId};
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

    #[cfg(feature = "telegram")]
    {
        use super::client;

        let (client, runner_handle) = client::create_client().await?;

        // Find the dialog by ID or name
        let mut dialogs = client.iter_dialogs();
        let mut found_dialog = None;

        while let Some(dialog) = dialogs.next().await? {
            let peer = dialog.peer();
            let name = peer.name().unwrap_or("");
            let peer_id = peer.id().bot_api_dialog_id().to_string();

            if peer_id == chat || name.to_lowercase().contains(&chat.to_lowercase()) {
                found_dialog = Some(dialog);
                break;
            }
        }

        let dialog = match found_dialog {
            Some(d) => d,
            None => {
                println!();
                println!("{}", format!("Chat not found: {}", chat).yellow());
                println!("  Use {} to see available chats", "chat telegram list".cyan());
                runner_handle.abort();
                return Ok(());
            }
        };

        let peer = dialog.peer();
        println!("  {}: {} (ID: {})", "Chat".dimmed(), peer.name().unwrap_or("Unknown"), peer.id().bot_api_dialog_id());

        // Fetch messages
        let mut messages = Vec::new();
        let mut msg_iter = client.iter_messages(peer);
        let mut count = 0;
        let max_limit = limit.unwrap_or(usize::MAX);

        while let Some(msg) = msg_iter.next().await? {
            // Apply time filters
            let msg_time = msg.date();

            if let Some(since_t) = since_time {
                if msg_time < since_t {
                    break; // Messages are in reverse chronological order
                }
            }

            if let Some(before_t) = before_time {
                if msg_time >= before_t {
                    continue;
                }
            }

            // Convert to our Message type
            let message = convert_message(&msg, &peer);
            messages.push(message);

            count += 1;
            if count >= max_limit {
                break;
            }

            // Show progress every 100 messages
            if count % 100 == 0 {
                print!("\r  {}: {}", "Fetched".dimmed(), count);
                use std::io::Write;
                std::io::stdout().flush()?;
            }
        }

        if count > 0 {
            println!("\r  {}: {}", "Fetched".dimmed(), count);
        }

        runner_handle.abort();

        if messages.is_empty() {
            println!();
            println!("{}", "No messages found.".yellow());
            return Ok(());
        }

        let formatted = formatters::format_messages(&messages, format)?;
        std::fs::write(&output, &formatted)
            .context("Failed to write output file")?;

        println!();
        println!("{}", format!("Exported {} messages to: {}", messages.len(), output).green());
    }

    #[cfg(not(feature = "telegram"))]
    {
        println!();
        println!("{}", "Note:".yellow().bold());
        println!("  The telegram feature is not enabled.");
        println!("  Build with: cargo build --features telegram");
    }

    Ok(())
}

#[cfg(feature = "telegram")]
fn convert_message(
    msg: &grammers_client::types::Message,
    peer: &grammers_client::types::Peer,
) -> Message {
    let id = MessageId::new(&msg.id().to_string());
    let chat_id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());

    let timestamp = msg.date();

    // Get sender info
    let sender = if let Ok(sender_peer) = msg.peer() {
        let sender_name = sender_peer.name().unwrap_or("Unknown");
        let sender_id = sender_peer.id().bot_api_dialog_id();

        // For outgoing messages, use "User" as display name
        let display_name = if msg.outgoing() {
            "User".to_string()
        } else {
            sender_name.to_string()
        };

        User {
            id: UserId::new(&sender_id.to_string()),
            username: None,
            display_name: Some(display_name),
            phone_number: None,
        }
    } else {
        User {
            id: UserId::new("unknown"),
            username: None,
            display_name: Some("Unknown".to_string()),
            phone_number: None,
        }
    };

    // Extract message content
    let content = if !msg.text().is_empty() {
        MessageContent::Text(msg.text().to_string())
    } else if msg.media().is_some() {
        MessageContent::Unknown
    } else {
        MessageContent::Text("".to_string())
    };

    let reply_to = msg.reply_to_message_id().map(|id| MessageId::new(&id.to_string()));

    Message {
        id,
        chat_id,
        sender,
        content,
        timestamp,
        reply_to,
        edited: msg.edit_date().is_some(),
    }
}
