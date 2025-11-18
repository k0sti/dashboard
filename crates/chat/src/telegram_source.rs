use anyhow::Result;
use async_trait::async_trait;

use crate::types::{
    Chat, ChatFilter, ChatId, ChatPattern, ChatSource, ChatType, Message,
    MessageContent, MessageFilter, MessageId, User, UserId,
};

#[cfg(feature = "telegram")]
use grammers_client::types::Peer;
#[cfg(feature = "telegram")]
use grammers_client::Client;
#[cfg(feature = "telegram")]
use grammers_mtsender::SenderPool;
#[cfg(feature = "telegram")]
use grammers_session::storages::SqliteSession;
#[cfg(feature = "telegram")]
use std::path::PathBuf;
#[cfg(feature = "telegram")]
use std::sync::Arc;
#[cfg(feature = "telegram")]
use tokio::task::JoinHandle;

/// Telegram chat source implementation
pub struct TelegramSource {
    #[cfg(feature = "telegram")]
    client: Option<Client>,
    #[cfg(feature = "telegram")]
    _runner_handle: Option<JoinHandle<()>>,
}

impl TelegramSource {
    /// Create a new Telegram source
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "telegram")]
            client: None,
            #[cfg(feature = "telegram")]
            _runner_handle: None,
        }
    }

    /// Connect to Telegram with the given API ID and session file path
    #[cfg(feature = "telegram")]
    pub async fn connect_with_session(&mut self, api_id: i32, session_path: PathBuf) -> Result<()> {
        let session_path_str = session_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid session path"))?;

        // Check if session file exists
        if !session_path.exists() {
            anyhow::bail!("Session file not found: {}", session_path.display());
        }

        // Load session
        let session = Arc::new(SqliteSession::open(session_path_str)?);

        // Create sender pool and client
        let pool = SenderPool::new(Arc::clone(&session), api_id);
        let client = Client::new(&pool);

        // Start the network runner
        let SenderPool { runner, .. } = pool;
        let runner_handle = tokio::spawn(runner.run());

        // Check if authorized
        if !client.is_authorized().await? {
            anyhow::bail!("Not authenticated. Session is invalid.");
        }

        self.client = Some(client);
        self._runner_handle = Some(runner_handle);

        Ok(())
    }

    /// Connect to Telegram (no-op when feature is disabled)
    #[cfg(not(feature = "telegram"))]
    pub async fn connect_with_session(&mut self, _api_id: i32, _session_path: std::path::PathBuf) -> Result<()> {
        anyhow::bail!("Telegram feature is not enabled");
    }

    /// Get the client reference
    #[cfg(feature = "telegram")]
    fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected. Call connect_with_session() first."))
    }
}

impl Default for TelegramSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChatSource for TelegramSource {
    fn source_id(&self) -> &str {
        "telegram"
    }

    fn source_name(&self) -> &str {
        "Telegram"
    }

    fn is_connected(&self) -> bool {
        #[cfg(feature = "telegram")]
        {
            self.client.is_some()
        }
        #[cfg(not(feature = "telegram"))]
        {
            false
        }
    }

    async fn list_chats(&self, filter: Option<ChatFilter>) -> Result<Vec<Chat>> {
        #[cfg(feature = "telegram")]
        {
            let client = self.client()?;
            let mut chats = Vec::new();
            let mut dialogs = client.iter_dialogs();

            while let Some(dialog) = dialogs.next().await? {
                let peer = dialog.peer();
                let chat = convert_peer_to_chat(&peer);

                // Apply filter if provided
                if let Some(ref filter) = filter {
                    if !filter.matches(&chat) {
                        continue;
                    }
                }

                chats.push(chat);
            }

            Ok(chats)
        }
        #[cfg(not(feature = "telegram"))]
        {
            anyhow::bail!("Telegram feature is not enabled");
        }
    }

    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>> {
        #[cfg(feature = "telegram")]
        {
            filter.validate()?;

            let client = self.client()?;
            let mut all_messages = Vec::new();

            // Determine which chats to query
            let chats_to_query = match &filter.chat {
                ChatPattern::Id(chat_id) => vec![chat_id.clone()],
                ChatPattern::Name(name) => {
                    // Find chats matching the name
                    let mut matched_chats = Vec::new();
                    let mut dialogs = client.iter_dialogs();

                    while let Some(dialog) = dialogs.next().await? {
                        let peer = dialog.peer();
                        let peer_name = peer.name().unwrap_or("");

                        if peer_name.to_lowercase().contains(&name.to_lowercase()) {
                            let chat_id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());
                            matched_chats.push(chat_id);
                        }
                    }

                    matched_chats
                }
                ChatPattern::All => {
                    // Get all chats
                    let mut all_chat_ids = Vec::new();
                    let mut dialogs = client.iter_dialogs();

                    while let Some(dialog) = dialogs.next().await? {
                        let peer = dialog.peer();
                        let chat_id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());
                        all_chat_ids.push(chat_id);
                    }

                    all_chat_ids
                }
                ChatPattern::Multiple(ids) => ids.clone(),
            };

            // Query messages from each chat
            for chat_id in chats_to_query {
                // Find the peer for this chat ID
                let mut dialogs = client.iter_dialogs();
                let mut found_peer: Option<Peer> = None;

                while let Some(dialog) = dialogs.next().await? {
                    let peer = dialog.peer();
                    let peer_id = peer.id().bot_api_dialog_id().to_string();

                    if peer_id == chat_id.as_str() {
                        found_peer = Some(peer.clone());
                        break;
                    }
                }

                if let Some(ref peer) = found_peer {
                    // Fetch messages from this peer
                    let mut msg_iter = client.iter_messages(peer);
                    let max_messages = filter.limit.unwrap_or(1000);
                    let mut count = 0;

                    while let Some(msg) = msg_iter.next().await? {
                        let message = convert_message(&msg, peer);

                        // Apply filters
                        if filter.matches(&message) {
                            all_messages.push(message);
                            count += 1;

                            if count >= max_messages {
                                break;
                            }
                        }
                    }
                }
            }

            // Sort by timestamp (most recent first)
            all_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            // Apply limit
            if let Some(limit) = filter.limit {
                all_messages.truncate(limit);
            }

            Ok(all_messages)
        }
        #[cfg(not(feature = "telegram"))]
        {
            anyhow::bail!("Telegram feature is not enabled");
        }
    }

    async fn subscribe(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>> {
        // Telegram streaming is supported but not implemented yet
        // This would use client.stream_updates()
        Ok(None)
    }
}

// Helper functions for Telegram-specific conversions
#[cfg(feature = "telegram")]
fn convert_peer_to_chat(peer: &Peer) -> Chat {
    let chat_id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());
    let title = peer.name().map(|s| s.to_string());

    // Note: grammers v0.8 API doesn't provide easy peer type discrimination
    // Setting to Unknown for now
    let chat_type = ChatType::Unknown;

    Chat {
        id: chat_id,
        title,
        chat_type,
        participant_count: None, // grammers doesn't easily provide this
    }
}

#[cfg(feature = "telegram")]
fn convert_message(msg: &grammers_client::types::Message, peer: &Peer) -> Message {
    let id = MessageId::new(&msg.id().to_string());
    let chat_id = ChatId::new(&peer.id().bot_api_dialog_id().to_string());
    let timestamp = msg.date();

    // Get sender info
    let sender = if let Some(sender_peer) = msg.sender() {
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

    let reply_to = msg
        .reply_to_message_id()
        .map(|id| MessageId::new(&id.to_string()));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_info() {
        let source = TelegramSource::new();
        assert_eq!(source.source_id(), "telegram");
        assert_eq!(source.source_name(), "Telegram");
        assert!(!source.is_connected());
    }

    #[test]
    fn test_default() {
        let source = TelegramSource::default();
        assert_eq!(source.source_id(), "telegram");
    }
}
