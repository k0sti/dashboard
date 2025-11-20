// ⚠️ WARNING: This uses an unofficial WhatsApp client library which may violate
// WhatsApp/Meta's Terms of Service. Using this code may result in temporary or
// permanent account suspension. Use at your own risk and only for personal/testing purposes.

use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::PathBuf;
use log::{info, warn, debug};

use crate::types::*;

#[cfg(feature = "whatsapp")]
use whatsapp_rust::Client;

/// WhatsApp source for unified chat API
///
/// MINIMAL VERTICAL SLICE - Only implements what's needed to:
/// - Authenticate via QR code
/// - List groups
/// - Get messages from a specific group
pub struct WhatsAppSource {
    #[cfg(feature = "whatsapp")]
    client: Option<Client>,
    session_path: PathBuf,
    connected: bool,
}

impl WhatsAppSource {
    /// Create a new WhatsApp source
    ///
    /// Session will be stored in the provided path
    pub fn new(session_path: PathBuf) -> Self {
        Self {
            #[cfg(feature = "whatsapp")]
            client: None,
            session_path,
            connected: false,
        }
    }

    /// Initialize connection with QR code authentication
    ///
    /// This will display a QR code in the terminal that needs to be scanned
    /// with the WhatsApp mobile app
    #[cfg(feature = "whatsapp")]
    pub async fn connect_with_qr(&mut self) -> Result<()> {
        info!("⚠️  WARNING: Using unofficial WhatsApp client - may violate ToS");
        info!("Initializing WhatsApp connection...");

        // Check if session exists
        if self.session_path.exists() {
            info!("Found existing session file at {:?}", self.session_path);
            self.load_session().await?;
        } else {
            warn!("No session found. QR code authentication required.");
            self.authenticate_with_qr().await?;
        }

        self.connected = true;
        Ok(())
    }

    #[cfg(feature = "whatsapp")]
    async fn load_session(&mut self) -> Result<()> {
        info!("Loading session from {:?}", self.session_path);

        // TODO: Implement actual session loading with whatsapp-rust
        // This is a placeholder - actual implementation depends on whatsapp-rust API

        bail!("Session loading not yet implemented - run with QR authentication first")
    }

    #[cfg(feature = "whatsapp")]
    async fn authenticate_with_qr(&mut self) -> Result<()> {
        info!("Starting QR code authentication...");

        // TODO: Implement QR code authentication
        // 1. Create WhatsApp client
        // 2. Generate QR code
        // 3. Display using qr2term
        // 4. Wait for scan
        // 5. Save session to session_path

        warn!("QR code authentication not yet fully implemented");
        warn!("This is a placeholder for the vertical slice");

        bail!("QR authentication requires full whatsapp-rust integration")
    }

    /// Find a group by name (case-insensitive partial match)
    #[cfg(feature = "whatsapp")]
    async fn find_group_by_name(&self, _name: &str) -> Result<Option<String>> {
        // TODO: Implement group search using whatsapp-rust types
        // Returns chat ID as string for now
        bail!("Group search not yet implemented")
    }

    /// Convert WhatsApp message to unified Message type
    #[cfg(feature = "whatsapp")]
    fn convert_message(&self, _msg_id: &str, chat_id: &str) -> Result<Message> {
        // TODO: Implement message conversion from whatsapp-rust types
        // This is a stub implementation for now

        Ok(Message {
            id: MessageId::new(format!("wa_{}", _msg_id)),
            chat_id: ChatId::new(chat_id),
            sender: User {
                id: UserId::new("unknown"),
                username: None,
                display_name: None,
                phone_number: None,
            },
            content: MessageContent::Text("TODO: Implement message conversion".to_string()),
            timestamp: chrono::Utc::now(),
            reply_to: None,
            edited: false,
        })
    }
}

#[async_trait]
impl ChatSource for WhatsAppSource {
    fn source_id(&self) -> &str {
        "whatsapp"
    }

    fn source_name(&self) -> &str {
        "WhatsApp"
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn list_chats(&self, filter: Option<ChatFilter>) -> Result<Vec<Chat>> {
        #[cfg(not(feature = "whatsapp"))]
        {
            bail!("WhatsApp support not enabled. Build with --features whatsapp")
        }

        #[cfg(feature = "whatsapp")]
        {
            if !self.connected {
                bail!("Not connected to WhatsApp. Run connect_with_qr() first.");
            }

            // TODO: Implement chat listing
            // 1. Get all chats from client
            // 2. Convert to unified Chat type
            // 3. Apply filters if provided

            warn!("list_chats not fully implemented yet");
            Ok(Vec::new())
        }
    }

    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>> {
        #[cfg(not(feature = "whatsapp"))]
        {
            bail!("WhatsApp support not enabled. Build with --features whatsapp")
        }

        #[cfg(feature = "whatsapp")]
        {
            if !self.connected {
                bail!("Not connected to WhatsApp. Run connect_with_qr() first.");
            }

            // Extract group name from filter
            let group_name = match &filter.chat {
                ChatPattern::Name(name) => name,
                ChatPattern::Id(id) => {
                    bail!("WhatsApp group lookup by ID not yet implemented. Use group name instead.");
                }
                ChatPattern::All => {
                    bail!("Fetching from all WhatsApp chats not supported. Specify a group name.");
                }
                ChatPattern::Multiple(_) => {
                    bail!("Multiple chat patterns not yet supported for WhatsApp.");
                }
            };

            info!("Fetching messages from WhatsApp group: {}", group_name);

            // TODO: Implement message fetching
            // 1. Find group by name
            // 2. Fetch messages from group
            // 3. Convert to unified Message type
            // 4. Apply filters (since, before, limit, search, sender)
            // 5. Return filtered messages

            warn!("get_messages not fully implemented yet - requires whatsapp-rust integration");
            warn!("This is a vertical slice placeholder");

            // Placeholder implementation
            let mut messages = Vec::new();

            // Apply limit if specified
            if let Some(limit) = filter.limit {
                messages.truncate(limit);
            }

            Ok(messages)
        }
    }

    async fn subscribe(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>> {
        // Real-time message streaming not needed for initial vertical slice
        Ok(None)
    }
}

/// Configuration for WhatsApp source
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhatsAppConfig {
    /// Path to session file
    pub session_path: PathBuf,
    /// Whether to auto-save session
    pub auto_save_session: bool,
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            session_path: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("chat")
                .join("whatsapp_session.bin"),
            auto_save_session: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatsapp_source_creation() {
        let session_path = PathBuf::from("/tmp/whatsapp_test_session");
        let source = WhatsAppSource::new(session_path.clone());

        assert_eq!(source.source_id(), "whatsapp");
        assert_eq!(source.source_name(), "WhatsApp");
        assert!(!source.is_connected());
    }

    #[test]
    fn test_whatsapp_config_default() {
        let config = WhatsAppConfig::default();
        assert!(config.auto_save_session);
        assert!(config.session_path.to_string_lossy().contains("whatsapp_session"));
    }
}
