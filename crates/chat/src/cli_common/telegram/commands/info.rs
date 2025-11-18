use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::OutputFormat;

pub async fn execute(
    chat: Option<String>,
    id: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let chat_id = chat.or(id).context("Chat name or ID is required")?;

    println!("{}", format!("Fetching information for '{}'...", chat_id).dimmed());

    #[cfg(feature = "telegram")]
    {
        use super::client;

        let (client, runner_handle) = client::create_client().await?;

        // Find the chat/dialog by searching through dialogs
        let mut dialogs = client.iter_dialogs();
        let mut found = None;

        while let Some(dialog) = dialogs.next().await? {
            let peer = dialog.peer();
            let name = peer.name().unwrap_or("");
            let peer_id = peer.id().bot_api_dialog_id().to_string();

            if name.to_lowercase().contains(&chat_id.to_lowercase()) || peer_id == chat_id {
                found = Some(dialog);
                break;
            }
        }

        match found {
            Some(dialog) => {
                let peer = dialog.peer();
                let name = peer.name().unwrap_or("Unknown");
                let peer_id = peer.id().bot_api_dialog_id();

                // Display info based on format
                match format {
                    OutputFormat::Json => {
                        let info = serde_json::json!({
                            "id": peer_id,
                            "name": name,
                        });
                        println!("{}", serde_json::to_string_pretty(&info)?);
                    }
                    _ => {
                        println!();
                        println!("{}", "Chat Information".bold());
                        println!("  {}: {}", "ID".bold(), peer_id);
                        println!("  {}: {}", "Name".bold(), name);
                    }
                }
            }
            None => {
                println!();
                println!("{}", format!("Chat not found: {}", chat_id).yellow());
                println!();
                println!("  Use {} to see available chats", "chat telegram list".cyan());
            }
        }

        runner_handle.abort();
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
