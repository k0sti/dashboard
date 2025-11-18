use anyhow::{Context, Result};
use chat::{ChatId, Message, MessageContent, MessageId, User, UserId};
use colored::Colorize;

use crate::cli::OutputFormat;
use crate::formatters;
use super::parse_time;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    chat: Option<String>,
    id: Option<String>,
    limit: usize,
    since: Option<String>,
    before: Option<String>,
    after: Option<String>,
    sender: Option<String>,
    _message_type: Option<String>,
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
    println!("  {}: {}", "Limit".dimmed(), limit);

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

            if peer_id == chat_id || name.to_lowercase().contains(&chat_id.to_lowercase()) {
                found_dialog = Some(dialog);
                break;
            }
        }

        let dialog = match found_dialog {
            Some(d) => d,
            None => {
                println!();
                println!("{}", format!("Chat not found: {}", chat_id).yellow());
                println!("  Use {} to see available chats", "chat telegram list".cyan());
                runner_handle.abort();
                return Ok(());
            }
        };

        let peer = dialog.peer();

        // Fetch messages
        let mut messages = Vec::new();
        let mut msg_iter = client.iter_messages(peer);
        let mut count = 0;

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

            // Apply sender filter
            if let Some(ref sender_name) = sender {
                if let Ok(sender_peer) = msg.peer() {
                    let name = sender_peer.name().unwrap_or("");
                    if !name.to_lowercase().contains(&sender_name.to_lowercase()) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Convert to our Message type
            let message = convert_message(&msg, &peer);
            messages.push(message);

            count += 1;
            if count >= limit {
                break;
            }
        }

        runner_handle.abort();

        if messages.is_empty() {
            println!();
            println!("{}", "No messages found.".yellow());
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
