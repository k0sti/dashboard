use anyhow::Result;
use colored::Colorize;
use chat::{Chat, ChatId, ChatType};

use crate::cli::{ChatTypeFilter, OutputFormat};
use crate::formatters;

pub async fn execute(format: OutputFormat, chat_type: Option<ChatTypeFilter>) -> Result<()> {
    println!("{}", "Fetching chat list...".dimmed());

    #[cfg(feature = "telegram")]
    {
        use super::client;

        let (client, runner_handle) = client::create_client().await?;

        // Fetch all dialogs
        let mut chats = Vec::new();
        let mut dialogs = client.iter_dialogs();

        while let Some(dialog) = dialogs.next().await? {
            let peer = dialog.peer();
            let name = peer.name().unwrap_or("Unknown");
            let id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());

            // Note: grammers v0.8 API doesn't provide easy peer type discrimination
            // Setting all to Unknown for simplicity
            let chat = Chat {
                id,
                title: Some(name.to_string()),
                chat_type: ChatType::Unknown,
                participant_count: None,
            };
            chats.push(chat);
        }

        runner_handle.abort();

        // Apply filter (all are Unknown type, so filter will match all or none)
        let filtered_chats: Vec<_> = if chat_type.is_some() {
            println!();
            println!("{}", "Note: Type filtering not yet implemented in grammers v0.8 integration".yellow());
            chats
        } else {
            chats
        };

        if filtered_chats.is_empty() {
            println!();
            println!("{}", "No chats found.".yellow());
            return Ok(());
        }

        let output = formatters::format_chats(&filtered_chats, format)?;
        println!("{}", output);
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
