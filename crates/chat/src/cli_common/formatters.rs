use anyhow::Result;
use chat::{Chat, ChatType, Message, MessageContent};
use colored::Colorize;
use serde_json;

use crate::cli::OutputFormat;

pub fn format_chats(chats: &[Chat], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => Ok(format_chats_text(chats)),
        OutputFormat::Json => {
            Ok(serde_json::to_string_pretty(chats)?)
        }
        OutputFormat::Csv => Ok(format_chats_csv(chats)),
        OutputFormat::Compact => Ok(format_chats_compact(chats)),
    }
}

pub fn format_messages(messages: &[Message], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => Ok(format_messages_text(messages)),
        OutputFormat::Json => {
            Ok(serde_json::to_string_pretty(messages)?)
        }
        OutputFormat::Csv => Ok(format_messages_csv(messages)),
        OutputFormat::Compact => Ok(format_messages_compact(messages)),
    }
}

fn format_chats_text(chats: &[Chat]) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}\n\n", "Available Chats:".bold()));

    for chat in chats {
        let chat_type = match chat.chat_type {
            ChatType::DirectMessage => "DM",
            ChatType::Group => "Group",
            ChatType::Channel => "Channel",
            ChatType::Unknown => "Unknown",
        };

        let title = chat.title.as_deref().unwrap_or("Untitled");
        let id = &chat.id;

        output.push_str(&format!(
            "  {} {} {}\n",
            format!("[{}]", chat_type).cyan(),
            title.green(),
            format!("({})", id).dimmed()
        ));

        if let Some(count) = chat.participant_count {
            output.push_str(&format!("    {} participants\n", count.to_string().dimmed()));
        }
    }

    output
}

fn format_chats_csv(chats: &[Chat]) -> String {
    let mut output = String::from("id,title,type,participant_count\n");

    for chat in chats {
        let chat_type = match chat.chat_type {
            ChatType::DirectMessage => "dm",
            ChatType::Group => "group",
            ChatType::Channel => "channel",
            ChatType::Unknown => "unknown",
        };

        output.push_str(&format!(
            "{},{},{},{}\n",
            chat.id,
            chat.title.as_deref().unwrap_or(""),
            chat_type,
            chat.participant_count.map(|c| c.to_string()).unwrap_or_default()
        ));
    }

    output
}

fn format_chats_compact(chats: &[Chat]) -> String {
    chats
        .iter()
        .map(|chat| {
            format!(
                "{} | {}",
                chat.id,
                chat.title.as_deref().unwrap_or("Untitled")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_messages_text(messages: &[Message]) -> String {
    let mut output = String::new();

    for msg in messages {
        let timestamp = msg.timestamp.format("%Y-%m-%d %H:%M:%S");
        let sender = msg.sender.username.as_deref()
            .or(msg.sender.display_name.as_deref())
            .unwrap_or("Unknown");

        output.push_str(&format!("{} ", format!("[{}]", timestamp).dimmed()));
        output.push_str(&format!("{}: ", sender.cyan()));

        match &msg.content {
            MessageContent::Text(text) => {
                output.push_str(text);
            }
            MessageContent::Image { caption, .. } => {
                output.push_str(&"[Image]".yellow().to_string());
                if let Some(cap) = caption {
                    output.push_str(&format!(" {}", cap));
                }
            }
            MessageContent::Video { caption, .. } => {
                output.push_str(&"[Video]".yellow().to_string());
                if let Some(cap) = caption {
                    output.push_str(&format!(" {}", cap));
                }
            }
            MessageContent::Audio { .. } => {
                output.push_str(&"[Audio]".yellow().to_string());
            }
            MessageContent::File { filename, .. } => {
                output.push_str(&format!("[File: {}]", filename.as_deref().unwrap_or("unknown")).yellow().to_string());
            }
            MessageContent::Sticker => {
                output.push_str(&"[Sticker]".yellow().to_string());
            }
            MessageContent::Location { latitude, longitude } => {
                output.push_str(&format!("[Location: {}, {}]", latitude, longitude).yellow().to_string());
            }
            MessageContent::Contact { name, phone } => {
                output.push_str(&format!(
                    "[Contact: {}{}]",
                    name,
                    phone.as_ref().map(|p| format!(" ({})", p)).unwrap_or_default()
                ).yellow().to_string());
            }
            MessageContent::Unknown => {
                output.push_str(&"[Unknown message type]".red().to_string());
            }
        }

        output.push('\n');
    }

    output
}

fn format_messages_csv(messages: &[Message]) -> String {
    let mut output = String::from("timestamp,sender,content_type,content\n");

    for msg in messages {
        let timestamp = msg.timestamp.to_rfc3339();
        let sender = msg.sender.username.as_deref()
            .or(msg.sender.display_name.as_deref())
            .unwrap_or("Unknown");

        let (content_type, content) = match &msg.content {
            MessageContent::Text(text) => ("text", text.clone()),
            MessageContent::Image { caption, .. } => ("image", caption.clone().unwrap_or_default()),
            MessageContent::Video { caption, .. } => ("video", caption.clone().unwrap_or_default()),
            MessageContent::Audio { .. } => ("audio", String::new()),
            MessageContent::File { filename, .. } => ("file", filename.clone().unwrap_or_default()),
            MessageContent::Sticker => ("sticker", String::new()),
            MessageContent::Location { latitude, longitude } => {
                ("location", format!("{},{}", latitude, longitude))
            }
            MessageContent::Contact { name, .. } => ("contact", name.clone()),
            MessageContent::Unknown => ("unknown", String::new()),
        };

        // Escape CSV content
        let content_escaped = content.replace('"', "\"\"");

        output.push_str(&format!(
            "{},{},{},\"{}\"\n",
            timestamp, sender, content_type, content_escaped
        ));
    }

    output
}

fn format_messages_compact(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|msg| {
            let sender = msg.sender.username.as_deref()
                .or(msg.sender.display_name.as_deref())
                .unwrap_or("Unknown");

            let content = match &msg.content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Image { .. } => "[Image]".to_string(),
                MessageContent::Video { .. } => "[Video]".to_string(),
                MessageContent::Audio { .. } => "[Audio]".to_string(),
                MessageContent::File { .. } => "[File]".to_string(),
                MessageContent::Sticker => "[Sticker]".to_string(),
                MessageContent::Location { .. } => "[Location]".to_string(),
                MessageContent::Contact { .. } => "[Contact]".to_string(),
                MessageContent::Unknown => "[Unknown]".to_string(),
            };

            format!("{} | {}: {}", msg.timestamp.format("%Y-%m-%d %H:%M"), sender, content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
