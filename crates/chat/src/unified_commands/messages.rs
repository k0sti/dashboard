use anyhow::Result;
use colored::Colorize;

use chat::{MessageFilter, SourcesManager, filter_parser};

pub async fn execute(
    filter: String,
    since: Option<String>,
    before: Option<String>,
    sender: Option<String>,
    search: Option<String>,
    limit: Option<usize>,
    format: String,
) -> Result<()> {
    println!("{}", "Querying messages...".dimmed());

    // Parse source:pattern filter
    let (source_id, chat_pattern) = filter_parser::parse_source_filter(&filter)?;

    // Create sources manager
    let manager = SourcesManager::new();

    // Build message filter
    let mut msg_filter = MessageFilter {
        chat: chat_pattern,
        since: None,
        before: None,
        sender,
        search,
        limit,
        content_type: None,
    };

    // Parse time specifications
    if let Some(since_spec) = since {
        msg_filter.since = Some(filter_parser::parse_time_spec(&since_spec)?);
    }

    if let Some(before_spec) = before {
        msg_filter.before = Some(filter_parser::parse_time_spec(&before_spec)?);
    }

    // Query messages
    let messages = manager.query_messages(source_id.as_deref(), msg_filter).await?;

    if messages.is_empty() {
        println!();
        println!("{}", "No messages found.".yellow());
        return Ok(());
    }

    // Format output
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&messages)?;
            println!("{}", json);
        }
        "csv" => {
            println!("ID,Chat ID,Sender,Timestamp,Content");
            for msg in messages {
                let sender_name = msg.sender.display_name.as_deref().unwrap_or("Unknown");
                let content = match &msg.content {
                    chat::MessageContent::Text(text) => text.replace('\n', " ").replace(',', ";"),
                    _ => "[Non-text content]".to_string(),
                };
                println!("{},{},{},{},{}",
                    msg.id, msg.chat_id, sender_name, msg.timestamp.to_rfc3339(), content);
            }
        }
        "compact" => {
            for msg in messages {
                let sender_name = msg.sender.display_name.as_deref().unwrap_or("Unknown");
                let content = match &msg.content {
                    chat::MessageContent::Text(text) => text,
                    _ => "[Non-text content]",
                };
                println!("[{}] {}: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S"), sender_name, content);
            }
        }
        "text" | _ => {
            println!();
            println!("{} {} messages found:", "Found".bold(), messages.len());
            println!();

            for msg in messages {
                let sender_name = msg.sender.display_name.as_deref().unwrap_or("Unknown");
                let timestamp = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed();

                println!("{} {} {}",
                    timestamp,
                    format!("{}:", sender_name).cyan().bold(),
                    ""
                );

                match &msg.content {
                    chat::MessageContent::Text(text) => {
                        for line in text.lines() {
                            println!("  {}", line);
                        }
                    }
                    chat::MessageContent::Image { caption, .. } => {
                        println!("  {} {}", "[Image]".yellow(), caption.as_deref().unwrap_or(""));
                    }
                    chat::MessageContent::Video { caption, .. } => {
                        println!("  {} {}", "[Video]".yellow(), caption.as_deref().unwrap_or(""));
                    }
                    chat::MessageContent::Audio { .. } => {
                        println!("  {}", "[Audio]".yellow());
                    }
                    chat::MessageContent::File { filename, .. } => {
                        println!("  {} {}", "[File]".yellow(), filename.as_deref().unwrap_or(""));
                    }
                    chat::MessageContent::Sticker => {
                        println!("  {}", "[Sticker]".yellow());
                    }
                    chat::MessageContent::Location { latitude, longitude } => {
                        println!("  {} {}, {}", "[Location]".yellow(), latitude, longitude);
                    }
                    chat::MessageContent::Contact { name, phone } => {
                        println!("  {} {} {}", "[Contact]".yellow(), name, phone.as_deref().unwrap_or(""));
                    }
                    chat::MessageContent::Unknown => {
                        println!("  {}", "[Unknown content]".dimmed());
                    }
                }

                println!();
            }
        }
    }

    Ok(())
}
