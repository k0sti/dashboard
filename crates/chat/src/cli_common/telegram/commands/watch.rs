use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::OutputFormat;

pub async fn execute(chat: Option<String>, all: bool, format: OutputFormat) -> Result<()> {
    if all {
        println!("{}", "Watching all chats for new messages...".bold());
    } else if let Some(ref chat_name) = chat {
        println!("{}", format!("Watching '{}' for new messages...", chat_name).bold());
    } else {
        anyhow::bail!("Either provide a chat name or use --all flag");
    }

    println!("{}", "Press Ctrl+C to stop watching.".dimmed());
    println!();

    #[cfg(feature = "telegram")]
    {
        use grammers_client::{Update, UpdatesConfiguration};
        use grammers_mtsender::SenderPool;
        use grammers_session::storages::SqliteSession;
        use std::sync::Arc;

        use crate::config::Config;

        let config = Config::load()?;

        let api_id = config
            .api_id
            .context("API ID not configured. Run 'chat telegram init'")?;

        // Get session file path
        let session_path = Config::session_file()?;
        let session_path_str = session_path
            .to_str()
            .context("Invalid session path")?;

        // Check if session file exists
        if !session_path.exists() {
            anyhow::bail!("Session not found. Run 'chat telegram init' to authenticate");
        }

        // Load session
        let session = Arc::new(SqliteSession::open(session_path_str)?);

        // Create sender pool and client
        let pool = SenderPool::new(Arc::clone(&session), api_id);
        let client = grammers_client::Client::new(&pool);

        // Extract components from pool
        let SenderPool {
            runner,
            updates,
            handle: _handle,
        } = pool;

        // Start the network runner
        let _runner_handle = tokio::spawn(runner.run());

        // Check if authorized
        if !client.is_authorized().await? {
            anyhow::bail!("Not authenticated. Run 'chat telegram init' to authenticate");
        }

        // Find the target peer if chat is specified
        let target_peer_id = if let Some(ref chat_id) = chat {
            let mut dialogs = client.iter_dialogs();
            let mut found_peer_id = None;

            while let Some(dialog) = dialogs.next().await? {
                let peer = dialog.peer();
                let name = peer.name().unwrap_or("");
                let peer_id = peer.id().bot_api_dialog_id();

                if peer_id.to_string() == *chat_id || name.to_lowercase().contains(&chat_id.to_lowercase()) {
                    found_peer_id = Some(peer_id);
                    println!("{}", format!("Found chat: {} (ID: {})", name, peer_id).green());
                    break;
                }
            }

            if found_peer_id.is_none() {
                println!();
                println!("{}", format!("Chat not found: {}", chat_id).yellow());
                println!("  Use {} to see available chats", "chat telegram list".cyan());
                return Ok(());
            }

            found_peer_id
        } else {
            None
        };

        println!();

        // Stream updates
        let mut updates = client.stream_updates(
            updates,
            UpdatesConfiguration {
                catch_up: false,
                ..Default::default()
            },
        );

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!();
                    println!("{}", "Stopping watch...".yellow());
                    break;
                }
                update = updates.next() => {
                    let update = update?;

                    match update {
                        Update::NewMessage(message) if !message.outgoing() => {
                            // Get message peer ID
                            let msg_peer_id = message.peer_id().bot_api_dialog_id();

                            // Filter by chat if specified
                            if let Some(target_id) = target_peer_id {
                                if msg_peer_id != target_id {
                                    continue;
                                }
                            }

                            // Get sender info
                            let sender_name = if let Ok(peer) = message.peer() {
                                peer.name().unwrap_or("Unknown").to_string()
                            } else {
                                "Unknown".to_string()
                            };

                            // Display message based on format
                            match format {
                                OutputFormat::Json => {
                                    let json_msg = serde_json::json!({
                                        "sender": sender_name,
                                        "chat_id": msg_peer_id,
                                        "text": message.text(),
                                        "timestamp": message.date().to_rfc3339(),
                                    });
                                    println!("{}", serde_json::to_string(&json_msg)?);
                                }
                                _ => {
                                    println!(
                                        "[{}] {}: {}",
                                        message.date().format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
                                        sender_name.cyan(),
                                        message.text()
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Sync update state before exiting
        updates.sync_update_state();
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
