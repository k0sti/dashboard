use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{ChatFilter, ChatType, MessageFilter};
use crate::filter_parser;

pub mod server;
pub mod tools;

pub use server::ChatMcpServer;

/// MCP tool names
pub const TOOL_LIST_SOURCES: &str = "list_sources";
pub const TOOL_LIST_CHATS: &str = "list_chats";
pub const TOOL_GET_MESSAGES: &str = "get_messages";

/// Request/Response types for MCP tools

#[derive(Debug, Deserialize)]
pub struct ListSourcesRequest {}

#[derive(Debug, Serialize)]
pub struct ListSourcesResponse {
    pub sources: Vec<SourceInfo>,
}

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
    pub is_connected: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListChatsRequest {
    pub source: String,
    #[serde(default)]
    pub name_pattern: Option<String>,
    #[serde(default)]
    pub chat_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListChatsResponse {
    pub chats: Vec<ChatInfo>,
}

#[derive(Debug, Serialize)]
pub struct ChatInfo {
    pub id: String,
    pub title: Option<String>,
    pub chat_type: String,
    pub participant_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesRequest {
    pub source: Option<String>,
    pub chat: String,
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub chat_id: String,
    pub sender: SenderInfo,
    pub content: String,
    pub timestamp: String,
    pub edited: bool,
}

#[derive(Debug, Serialize)]
pub struct SenderInfo {
    pub id: String,
    pub display_name: Option<String>,
}

/// Convert internal types to MCP response types
impl From<crate::types::SourceInfo> for SourceInfo {
    fn from(info: crate::types::SourceInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            is_connected: info.is_connected,
        }
    }
}

impl From<&crate::types::Chat> for ChatInfo {
    fn from(chat: &crate::types::Chat) -> Self {
        let chat_type = match chat.chat_type {
            ChatType::DirectMessage => "direct".to_string(),
            ChatType::Group => "group".to_string(),
            ChatType::Channel => "channel".to_string(),
            ChatType::Unknown => "unknown".to_string(),
        };

        Self {
            id: chat.id.to_string(),
            title: chat.title.clone(),
            chat_type,
            participant_count: chat.participant_count,
        }
    }
}

impl From<&crate::types::Message> for MessageInfo {
    fn from(msg: &crate::types::Message) -> Self {
        let content = match &msg.content {
            crate::types::MessageContent::Text(text) => text.clone(),
            crate::types::MessageContent::Image { caption, .. } => {
                format!("[Image] {}", caption.as_deref().unwrap_or(""))
            }
            crate::types::MessageContent::Video { caption, .. } => {
                format!("[Video] {}", caption.as_deref().unwrap_or(""))
            }
            crate::types::MessageContent::Audio { .. } => "[Audio]".to_string(),
            crate::types::MessageContent::File { filename, .. } => {
                format!("[File] {}", filename.as_deref().unwrap_or(""))
            }
            crate::types::MessageContent::Sticker => "[Sticker]".to_string(),
            crate::types::MessageContent::Location { latitude, longitude } => {
                format!("[Location] {}, {}", latitude, longitude)
            }
            crate::types::MessageContent::Contact { name, phone } => {
                format!("[Contact] {} {}", name, phone.as_deref().unwrap_or(""))
            }
            crate::types::MessageContent::Unknown => "[Unknown]".to_string(),
        };

        Self {
            id: msg.id.to_string(),
            chat_id: msg.chat_id.to_string(),
            sender: SenderInfo {
                id: msg.sender.id.to_string(),
                display_name: msg.sender.display_name.clone(),
            },
            content,
            timestamp: msg.timestamp.to_rfc3339(),
            edited: msg.edited,
        }
    }
}

/// Helper function to parse chat type string
pub fn parse_chat_type(type_str: &str) -> Result<ChatType> {
    match type_str.to_lowercase().as_str() {
        "direct" | "dm" => Ok(ChatType::DirectMessage),
        "group" => Ok(ChatType::Group),
        "channel" => Ok(ChatType::Channel),
        _ => anyhow::bail!("Invalid chat type '{}'. Expected: direct, group, channel", type_str),
    }
}

/// Build a ChatFilter from ListChatsRequest
pub fn build_chat_filter(req: &ListChatsRequest) -> Result<Option<ChatFilter>> {
    let mut filter = ChatFilter::new();
    let mut has_filters = false;

    if let Some(ref name_pattern) = req.name_pattern {
        filter = filter.with_name(name_pattern);
        has_filters = true;
    }

    if let Some(ref type_str) = req.chat_type {
        let chat_type = parse_chat_type(type_str)?;
        filter = filter.with_type(chat_type);
        has_filters = true;
    }

    if has_filters {
        Ok(Some(filter))
    } else {
        Ok(None)
    }
}

/// Build a MessageFilter from GetMessagesRequest
pub async fn build_message_filter(req: &GetMessagesRequest) -> Result<MessageFilter> {
    // Parse chat pattern from the chat field
    let (_, chat_pattern) = filter_parser::parse_source_filter(&req.chat)?;

    let mut filter = MessageFilter {
        chat: chat_pattern,
        since: None,
        before: None,
        sender: req.sender.clone(),
        search: req.search.clone(),
        limit: req.limit,
        content_type: None,
    };

    // Parse time specifications
    if let Some(ref since_spec) = req.since {
        filter.since = Some(filter_parser::parse_time_spec(since_spec)?);
    }

    if let Some(ref before_spec) = req.before {
        filter.before = Some(filter_parser::parse_time_spec(before_spec)?);
    }

    // Validate filter
    filter.validate()?;

    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chat_type() {
        assert!(matches!(parse_chat_type("direct").unwrap(), ChatType::DirectMessage));
        assert!(matches!(parse_chat_type("dm").unwrap(), ChatType::DirectMessage));
        assert!(matches!(parse_chat_type("group").unwrap(), ChatType::Group));
        assert!(matches!(parse_chat_type("channel").unwrap(), ChatType::Channel));
        assert!(parse_chat_type("invalid").is_err());
    }

    #[test]
    fn test_build_chat_filter_empty() {
        let req = ListChatsRequest {
            source: "telegram".to_string(),
            name_pattern: None,
            chat_type: None,
        };
        let filter = build_chat_filter(&req).unwrap();
        assert!(filter.is_none());
    }

    #[test]
    fn test_build_chat_filter_with_name() {
        let req = ListChatsRequest {
            source: "telegram".to_string(),
            name_pattern: Some("Work".to_string()),
            chat_type: None,
        };
        let filter = build_chat_filter(&req).unwrap();
        assert!(filter.is_some());
    }
}
