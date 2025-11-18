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
/// DEPRECATED: Use ChatSource instead for new code
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

// ============================================================================
// NEW UNIFIED API - ChatSource and Filters
// ============================================================================

/// Pattern for matching chats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatPattern {
    /// Specific chat by ID
    Id(ChatId),
    /// Chat by name (partial match, case-insensitive)
    Name(String),
    /// All chats
    All,
    /// Multiple specific chats
    Multiple(Vec<ChatId>),
}

impl ChatPattern {
    /// Check if this pattern matches a chat
    pub fn matches(&self, chat: &Chat) -> bool {
        match self {
            ChatPattern::Id(id) => &chat.id == id,
            ChatPattern::Name(name) => {
                if let Some(title) = &chat.title {
                    title.to_lowercase().contains(&name.to_lowercase())
                } else {
                    false
                }
            }
            ChatPattern::All => true,
            ChatPattern::Multiple(ids) => ids.contains(&chat.id),
        }
    }
}

/// Content type filter for messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Video,
    Audio,
    File,
    Sticker,
    Location,
    Contact,
}

impl ContentType {
    /// Check if this content type matches message content
    pub fn matches(&self, content: &MessageContent) -> bool {
        match (self, content) {
            (ContentType::Text, MessageContent::Text(_)) => true,
            (ContentType::Image, MessageContent::Image { .. }) => true,
            (ContentType::Video, MessageContent::Video { .. }) => true,
            (ContentType::Audio, MessageContent::Audio { .. }) => true,
            (ContentType::File, MessageContent::File { .. }) => true,
            (ContentType::Sticker, MessageContent::Sticker) => true,
            (ContentType::Location, MessageContent::Location { .. }) => true,
            (ContentType::Contact, MessageContent::Contact { .. }) => true,
            _ => false,
        }
    }
}

/// Filter for querying messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    /// Source-specific chat ID or name pattern
    pub chat: ChatPattern,
    /// Time range - messages after this time
    pub since: Option<DateTime<Utc>>,
    /// Time range - messages before this time
    pub before: Option<DateTime<Utc>>,
    /// Sender filter (name or ID pattern)
    pub sender: Option<String>,
    /// Text search (case-insensitive substring)
    pub search: Option<String>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Message content types
    pub content_type: Option<Vec<ContentType>>,
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self {
            chat: ChatPattern::All,
            since: None,
            before: None,
            sender: None,
            search: None,
            limit: Some(100),
            content_type: None,
        }
    }
}

impl MessageFilter {
    /// Create a new filter for all chats
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter for a specific chat by ID
    pub fn for_chat_id(chat_id: ChatId) -> Self {
        Self {
            chat: ChatPattern::Id(chat_id),
            ..Default::default()
        }
    }

    /// Create a filter for a chat by name pattern
    pub fn for_chat_name(name: impl Into<String>) -> Self {
        Self {
            chat: ChatPattern::Name(name.into()),
            ..Default::default()
        }
    }

    /// Validate the filter
    pub fn validate(&self) -> Result<()> {
        // Check time range
        if let (Some(since), Some(before)) = (&self.since, &self.before) {
            if since >= before {
                anyhow::bail!("since must be before before");
            }
        }

        // Check limit
        if let Some(limit) = self.limit {
            if limit == 0 {
                anyhow::bail!("limit must be positive");
            }
        }

        Ok(())
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &Message) -> bool {
        // Check time range
        if let Some(since) = &self.since {
            if &message.timestamp < since {
                return false;
            }
        }
        if let Some(before) = &self.before {
            if &message.timestamp >= before {
                return false;
            }
        }

        // Check sender
        if let Some(sender_pattern) = &self.sender {
            let sender_match = message.sender.display_name
                .as_ref()
                .map(|name| name.to_lowercase().contains(&sender_pattern.to_lowercase()))
                .unwrap_or(false)
                || message.sender.username
                    .as_ref()
                    .map(|username| username.to_lowercase().contains(&sender_pattern.to_lowercase()))
                    .unwrap_or(false);

            if !sender_match {
                return false;
            }
        }

        // Check text search
        if let Some(search_term) = &self.search {
            let text_match = match &message.content {
                MessageContent::Text(text) => {
                    text.to_lowercase().contains(&search_term.to_lowercase())
                }
                MessageContent::Image { caption: Some(caption), .. } |
                MessageContent::Video { caption: Some(caption), .. } => {
                    caption.to_lowercase().contains(&search_term.to_lowercase())
                }
                _ => false,
            };

            if !text_match {
                return false;
            }
        }

        // Check content type
        if let Some(content_types) = &self.content_type {
            if !content_types.iter().any(|ct| ct.matches(&message.content)) {
                return false;
            }
        }

        true
    }
}

/// Filter for listing chats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatFilter {
    /// Filter by chat type
    pub chat_type: Option<ChatType>,
    /// Name pattern matching (case-insensitive substring)
    pub name_pattern: Option<String>,
    /// Only include chats with recent activity
    pub active_since: Option<DateTime<Utc>>,
}

impl ChatFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by chat type
    pub fn with_type(mut self, chat_type: ChatType) -> Self {
        self.chat_type = Some(chat_type);
        self
    }

    /// Filter by name pattern
    pub fn with_name(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    /// Check if a chat matches this filter
    pub fn matches(&self, chat: &Chat) -> bool {
        // Check chat type
        if let Some(ref filter_type) = self.chat_type {
            if &chat.chat_type != filter_type {
                return false;
            }
        }

        // Check name pattern
        if let Some(ref pattern) = self.name_pattern {
            if let Some(ref title) = chat.title {
                if !title.to_lowercase().contains(&pattern.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Note: active_since requires additional data not in Chat struct
        // This would need to be checked by the implementation

        true
    }
}

/// Information about a chat source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Unique identifier (telegram, signal, whatsapp)
    pub id: String,
    /// Display name (Telegram, Signal, WhatsApp)
    pub name: String,
    /// Connection status
    pub is_connected: bool,
}

/// Unified chat source interface
#[async_trait::async_trait]
pub trait ChatSource: Send + Sync {
    /// Get source identifier (telegram, signal, whatsapp)
    fn source_id(&self) -> &str;

    /// Get display name for the source
    fn source_name(&self) -> &str;

    /// Check if source is connected
    fn is_connected(&self) -> bool;

    /// List all chats (conversations) from this source
    async fn list_chats(&self, filter: Option<ChatFilter>) -> Result<Vec<Chat>>;

    /// Get messages matching filter
    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>>;

    /// Subscribe to new messages (optional)
    /// Returns None if the source doesn't support streaming
    async fn subscribe(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>>;
}
