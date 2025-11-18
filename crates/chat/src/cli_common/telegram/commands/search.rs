use anyhow::Result;
use chat::{ChatId, Message, MessageContent, MessageId, User, UserId};
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

    #[cfg(feature = "telegram")]
    {
        use super::client;

        let (client, runner_handle) = client::create_client().await?;

        let mut all_messages = Vec::new();

        if all {
            // Search across all chats
            let mut dialogs = client.iter_dialogs();
            let mut chat_count = 0;

            while let Some(dialog) = dialogs.next().await? {
                let peer = dialog.peer();
                let messages = search_in_peer(&client, &peer, &term, ignore_case).await?;

                if !messages.is_empty() {
                    chat_count += 1;
                    all_messages.extend(messages);
                }
            }

            println!("  {}: {} chats", "Searched".dimmed(), chat_count);
        } else {
            // Search in specific chat
            let chat_id = chat.unwrap();
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
            all_messages = search_in_peer(&client, &peer, &term, ignore_case).await?;
        }

        runner_handle.abort();

        if all_messages.is_empty() {
            println!();
            println!("{}", "No messages found matching the search term.".yellow());
            return Ok(());
        }

        println!("  {}: {}", "Found".dimmed(), all_messages.len());
        println!();

        let formatted = formatters::format_messages(&all_messages, format)?;
        println!("{}", formatted);
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
async fn search_in_peer(
    client: &grammers_client::Client,
    peer: &grammers_client::types::Peer,
    term: &str,
    ignore_case: bool,
) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    let mut msg_iter = client.iter_messages(peer);
    let max_messages = 1000; // Limit search to last 1000 messages per chat
    let mut count = 0;

    let search_term = if ignore_case {
        term.to_lowercase()
    } else {
        term.to_string()
    };

    while let Some(msg) = msg_iter.next().await? {
        let text = msg.text();

        let matches = if ignore_case {
            text.to_lowercase().contains(&search_term)
        } else {
            text.contains(&search_term)
        };

        if matches {
            let message = convert_message(&msg, peer);
            messages.push(message);
        }

        count += 1;
        if count >= max_messages {
            break;
        }
    }

    Ok(messages)
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
