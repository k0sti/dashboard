use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a chat client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChatClientId(Uuid);

impl ChatClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ChatClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ChatClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported chat platforms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatPlatform {
    Telegram,
    WhatsApp,
    Signal,
    Matrix, // For mautrix-based unified access
}

impl fmt::Display for ChatPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatPlatform::Telegram => write!(f, "Telegram"),
            ChatPlatform::WhatsApp => write!(f, "WhatsApp"),
            ChatPlatform::Signal => write!(f, "Signal"),
            ChatPlatform::Matrix => write!(f, "Matrix"),
        }
    }
}

/// Connection status of a chat client
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatClientStatus {
    Disconnected,
    Connecting,
    Connected,
    Syncing,
    Error(String),
}

/// Configuration for a chat client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatClientConfig {
    pub id: ChatClientId,
    pub name: String,
    pub platform: ChatPlatform,
    pub config_data: serde_json::Value,
}

/// Unique identifier for a chat/conversation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChatId(String);

impl ChatId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ChatId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ChatId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a user
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(String);

impl MessageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MessageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MessageId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Information about a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub phone_number: Option<String>,
}

/// Type of chat
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatType {
    DirectMessage,
    Group,
    Channel,
    Unknown,
}

/// Information about a chat/conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: ChatId,
    pub title: Option<String>,
    pub chat_type: ChatType,
    pub participant_count: Option<usize>,
}

/// Content type of a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image { caption: Option<String>, url: Option<String> },
    Video { caption: Option<String>, url: Option<String> },
    Audio { url: Option<String> },
    File { filename: Option<String>, url: Option<String> },
    Sticker,
    Location { latitude: f64, longitude: f64 },
    Contact { name: String, phone: Option<String> },
    Unknown,
}

/// A message in a chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub chat_id: ChatId,
    pub sender: User,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<MessageId>,
    pub edited: bool,
}

/// Options for fetching messages
#[derive(Debug, Clone)]
pub struct MessageFetchOptions {
    /// Maximum number of messages to fetch
    pub limit: Option<usize>,
    /// Fetch messages before this message ID (for pagination)
    pub before: Option<MessageId>,
    /// Fetch messages after this message ID (for pagination)
    pub after: Option<MessageId>,
    /// Only fetch messages after this timestamp
    pub since: Option<DateTime<Utc>>,
}

impl Default for MessageFetchOptions {
    fn default() -> Self {
        Self {
            limit: Some(100),
            before: None,
            after: None,
            since: None,
        }
    }
}

/// Generic trait for chat clients (read-only)
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    /// Get the client configuration
    fn get_config(&self) -> &ChatClientConfig;

    /// Get the current connection status
    fn get_status(&self) -> ChatClientStatus;

    /// Connect to the chat platform
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the chat platform
    async fn disconnect(&mut self) -> Result<()>;

    /// List all available chats/conversations
    async fn list_chats(&self) -> Result<Vec<Chat>>;

    /// Get messages from a specific chat
    async fn get_messages(&self, chat_id: &ChatId, options: MessageFetchOptions) -> Result<Vec<Message>>;

    /// Get a specific message by ID
    async fn get_message(&self, chat_id: &ChatId, message_id: &MessageId) -> Result<Option<Message>>;

    /// Subscribe to new messages (returns a stream/channel)
    /// This is optional and can return None if the platform doesn't support streaming
    async fn subscribe_messages(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>>;
}
