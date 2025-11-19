use anyhow::Result;
use colored::Colorize;

use chat::{ChatFilter, ChatType, SourcesManager};

pub async fn execute(
    source: String,
    name: Option<String>,
    chat_type: Option<String>,
    format: String,
) -> Result<()> {
    println!("{}", format!("Listing chats from source '{}'...", source).dimmed());

    // Create sources manager
    let manager = SourcesManager::new();

    // Check if source exists
    if !manager.has_source(&source) {
        println!();
        println!("{}", format!("Source '{}' not found.", source).red());
        println!();
        println!("Available sources:");
        let sources = manager.list_sources()?;
        if sources.is_empty() {
            println!("  {}", "No sources configured.".yellow());
            println!();
            println!("Run {} to configure Telegram", "chat telegram init".cyan());
        } else {
            for s in sources {
                println!("  {} {}", "•".cyan(), s.id);
            }
        }
        println!();
        return Ok(());
    }

    // Build filter
    let mut filter = ChatFilter::new();

    if let Some(name_pattern) = name {
        filter = filter.with_name(name_pattern);
    }

    if let Some(type_str) = chat_type {
        let ct = match type_str.to_lowercase().as_str() {
            "direct" | "dm" => ChatType::DirectMessage,
            "group" => ChatType::Group,
            "channel" => ChatType::Channel,
            _ => {
                println!("{}", format!("Invalid chat type '{}'. Expected: direct, group, channel", type_str).red());
                return Ok(());
            }
        };
        filter = filter.with_type(ct);
    }

    // List chats
    let chats = manager.list_chats(&source, Some(filter)).await?;

    if chats.is_empty() {
        println!();
        println!("{}", "No chats found.".yellow());
        return Ok(());
    }

    // Format output
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&chats)?;
            println!("{}", json);
        }
        "csv" => {
            println!("ID,Name,Type,Participants");
            for chat in chats {
                let name = chat.title.as_deref().unwrap_or("Unknown");
                let chat_type = format!("{:?}", chat.chat_type);
                let participants = chat.participant_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                println!("{},{},{},{}", chat.id, name, chat_type, participants);
            }
        }
        "compact" => {
            for chat in chats {
                let name = chat.title.as_deref().unwrap_or("Unknown");
                println!("{} - {}", chat.id, name);
            }
        }
        "text" | _ => {
            println!();
            println!("{} {} chats found:", chats.len(), source);
            println!();

            for chat in chats {
                let name = chat.title.as_deref().unwrap_or("Unknown");
                let type_str = match chat.chat_type {
                    ChatType::DirectMessage => "Direct".cyan(),
                    ChatType::Group => "Group".green(),
                    ChatType::Channel => "Channel".yellow(),
                    ChatType::Unknown => "Unknown".dimmed(),
                };

                println!("  {} {} [{}]",
                    "•".cyan(),
                    name.bold(),
                    type_str
                );
                println!("    {}: {}", "ID".dimmed(), chat.id);

                if let Some(count) = chat.participant_count {
                    println!("    {}: {}", "Participants".dimmed(), count);
                }
            }

            println!();
        }
    }

    Ok(())
}
