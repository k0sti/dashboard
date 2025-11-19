use anyhow::Result;
use chat::{
    types::*, ChatSource, MessageFilter, SourcesManager, ChatFilter, ChatPattern,
};
use chrono::{DateTime, Utc, Duration};

#[cfg(feature = "mcp")]
use chat::mcp_server::{
    GetMessagesRequest, ListChatsRequest, ListSourcesRequest,
    tools::*,
};

/// Mock chat source for testing
struct MockChatSource {
    id: String,
    name: String,
    connected: bool,
    chats: Vec<Chat>,
    messages: Vec<Message>,
}

impl MockChatSource {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            connected: true,
            chats: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn with_chats(mut self, chats: Vec<Chat>) -> Self {
        self.chats = chats;
        self
    }

    fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }
}

#[async_trait::async_trait]
impl ChatSource for MockChatSource {
    fn source_id(&self) -> &str {
        &self.id
    }

    fn source_name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn list_chats(&self, filter: Option<ChatFilter>) -> Result<Vec<Chat>> {
        let mut chats = self.chats.clone();

        if let Some(filter) = filter {
            // Apply name filter
            if let Some(name_pattern) = &filter.name_pattern {
                let pattern_lower = name_pattern.to_lowercase();
                chats.retain(|chat| {
                    chat.title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&pattern_lower))
                        .unwrap_or(false)
                });
            }

            // Apply type filter
            if let Some(chat_type) = &filter.chat_type {
                chats.retain(|chat| chat.chat_type == *chat_type);
            }
        }

        Ok(chats)
    }

    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>> {
        let mut messages = self.messages.clone();

        // Apply chat pattern filter
        match &filter.chat {
            ChatPattern::Id(id) => {
                messages.retain(|msg| &msg.chat_id == id);
            }
            ChatPattern::Name(name) => {
                let name_lower = name.to_lowercase();
                let matching_chats: Vec<_> = self
                    .chats
                    .iter()
                    .filter(|chat| {
                        chat.title
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(&name_lower))
                            .unwrap_or(false)
                    })
                    .map(|chat| &chat.id)
                    .collect();

                messages.retain(|msg| matching_chats.contains(&&msg.chat_id));
            }
            ChatPattern::All => {
                // No filtering
            }
            ChatPattern::Multiple(_) => {
                // Not implemented for mock
            }
        }

        // Apply time filters
        if let Some(since) = filter.since {
            messages.retain(|msg| msg.timestamp >= since);
        }

        if let Some(before) = filter.before {
            messages.retain(|msg| msg.timestamp <= before);
        }

        // Apply sender filter
        if let Some(ref sender) = filter.sender {
            let sender_lower = sender.to_lowercase();
            messages.retain(|msg| {
                msg.sender
                    .display_name
                    .as_ref()
                    .map(|name| name.to_lowercase().contains(&sender_lower))
                    .unwrap_or(false)
                    || msg.sender.id.to_string() == *sender
            });
        }

        // Apply search filter
        if let Some(ref search) = filter.search {
            let search_lower = search.to_lowercase();
            messages.retain(|msg| match &msg.content {
                MessageContent::Text(text) => text.to_lowercase().contains(&search_lower),
                _ => false,
            });
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            messages.truncate(limit);
        }

        Ok(messages)
    }

    async fn subscribe(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>> {
        Ok(None)
    }
}

/// Helper to create a mock message
fn create_message(
    id: i64,
    chat_id: i64,
    sender_id: i64,
    sender_name: &str,
    content: &str,
    timestamp: DateTime<Utc>,
) -> Message {
    Message {
        id: MessageId::new(id.to_string()),
        chat_id: ChatId::new(chat_id.to_string()),
        sender: User {
            id: UserId::new(sender_id.to_string()),
            username: None,
            display_name: Some(sender_name.to_string()),
            phone_number: None,
        },
        content: MessageContent::Text(content.to_string()),
        timestamp,
        reply_to: None,
        edited: false,
    }
}

/// Helper to create a mock chat
fn create_chat(id: i64, title: &str, chat_type: ChatType) -> Chat {
    Chat {
        id: ChatId::new(id.to_string()),
        title: Some(title.to_string()),
        chat_type,
        participant_count: None,
    }
}

#[tokio::test]
async fn test_sources_manager_registration() -> Result<()> {
    let manager = SourcesManager::new();

    // Register a mock source
    let source = MockChatSource::new("test", "Test Source");
    manager.register(Box::new(source))?;

    // Verify it's listed
    let sources = manager.list_sources()?;
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].id, "test");
    assert_eq!(sources[0].name, "Test Source");
    assert!(sources[0].is_connected);

    Ok(())
}

#[tokio::test]
async fn test_cross_source_queries() -> Result<()> {
    let manager = SourcesManager::new();

    let now = Utc::now();

    // Create two sources with different chats and messages
    let source1_chats = vec![
        create_chat(1, "Work Chat", ChatType::Group),
        create_chat(2, "Personal", ChatType::DirectMessage),
    ];
    let source1_messages = vec![
        create_message(1, 1, 10, "Alice", "Hello from source1", now - Duration::hours(1)),
        create_message(2, 2, 11, "Bob", "Hi there", now - Duration::hours(2)),
    ];

    let source2_chats = vec![
        create_chat(3, "Work Chat", ChatType::Group),
        create_chat(4, "Friends", ChatType::Group),
    ];
    let source2_messages = vec![
        create_message(3, 3, 20, "Charlie", "Hello from source2", now - Duration::hours(1)),
        create_message(4, 4, 21, "Diana", "Hey everyone", now - Duration::hours(3)),
    ];

    let source1 = MockChatSource::new("source1", "Source 1")
        .with_chats(source1_chats)
        .with_messages(source1_messages);

    let source2 = MockChatSource::new("source2", "Source 2")
        .with_chats(source2_chats)
        .with_messages(source2_messages);

    manager.register(Box::new(source1))?;
    manager.register(Box::new(source2))?;

    // Query all messages from all sources
    let filter = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let messages = manager.query_messages(None, filter).await?;
    assert_eq!(messages.len(), 4, "Should get messages from both sources");

    // Query messages from specific source
    let filter2 = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let messages2 = manager.query_messages(Some("source1"), filter2).await?;
    assert_eq!(messages2.len(), 2, "Should get messages only from source1");

    Ok(())
}

#[tokio::test]
async fn test_message_filtering() -> Result<()> {
    let manager = SourcesManager::new();

    let now = Utc::now();

    let chats = vec![
        create_chat(1, "Work Chat", ChatType::Group),
        create_chat(2, "Personal", ChatType::DirectMessage),
    ];

    let messages = vec![
        create_message(1, 1, 10, "Alice", "Meeting tomorrow", now - Duration::hours(1)),
        create_message(2, 1, 11, "Bob", "Sounds good", now - Duration::hours(2)),
        create_message(3, 2, 12, "Charlie", "Hello world", now - Duration::hours(3)),
        create_message(4, 2, 13, "Diana", "Meeting canceled", now - Duration::days(2)),
    ];

    let source = MockChatSource::new("test", "Test")
        .with_chats(chats)
        .with_messages(messages);

    manager.register(Box::new(source))?;

    // Test search filter
    let filter = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: Some("meeting".to_string()),
        limit: None,
        content_type: None,
    };

    let results = manager.query_messages(Some("test"), filter).await?;
    assert_eq!(results.len(), 2, "Should find 2 messages with 'meeting'");

    // Test time filter
    let filter2 = MessageFilter {
        chat: ChatPattern::All,
        since: Some(now - Duration::hours(4)),
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let results2 = manager.query_messages(Some("test"), filter2).await?;
    assert_eq!(results2.len(), 3, "Should find 3 messages from last 4 hours");

    // Test sender filter
    let filter3 = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: Some("Alice".to_string()),
        search: None,
        limit: None,
        content_type: None,
    };

    let results3 = manager.query_messages(Some("test"), filter3).await?;
    assert_eq!(results3.len(), 1, "Should find 1 message from Alice");

    // Test limit
    let filter4 = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: Some(2),
        content_type: None,
    };

    let results4 = manager.query_messages(Some("test"), filter4).await?;
    assert_eq!(results4.len(), 2, "Should limit to 2 messages");

    Ok(())
}

#[tokio::test]
async fn test_chat_pattern_matching() -> Result<()> {
    let manager = SourcesManager::new();

    let now = Utc::now();

    let chats = vec![
        create_chat(1, "Work Team", ChatType::Group),
        create_chat(2, "Work Project", ChatType::Group),
        create_chat(3, "Personal", ChatType::DirectMessage),
    ];

    let messages = vec![
        create_message(1, 1, 10, "Alice", "Message 1", now),
        create_message(2, 2, 11, "Bob", "Message 2", now),
        create_message(3, 3, 12, "Charlie", "Message 3", now),
    ];

    let source = MockChatSource::new("test", "Test")
        .with_chats(chats)
        .with_messages(messages);

    manager.register(Box::new(source))?;

    // Test name pattern matching
    let filter = MessageFilter {
        chat: ChatPattern::Name("Work".to_string()),
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let results = manager.query_messages(Some("test"), filter).await?;
    assert_eq!(results.len(), 2, "Should match both 'Work' chats");

    // Test ID pattern matching
    let filter2 = MessageFilter {
        chat: ChatPattern::Id(ChatId::new("1")),
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let results2 = manager.query_messages(Some("test"), filter2).await?;
    assert_eq!(results2.len(), 1, "Should match chat ID 1");

    Ok(())
}

#[tokio::test]
async fn test_chat_filtering() -> Result<()> {
    let manager = SourcesManager::new();

    let chats = vec![
        create_chat(1, "Work Team", ChatType::Group),
        create_chat(2, "Personal Chat", ChatType::DirectMessage),
        create_chat(3, "News Channel", ChatType::Channel),
        create_chat(4, "Work Project", ChatType::Group),
    ];

    let source = MockChatSource::new("test", "Test").with_chats(chats);

    manager.register(Box::new(source))?;

    // Test type filter
    let filter = ChatFilter::new().with_type(ChatType::Group);
    let results = manager.list_chats("test", Some(filter)).await?;
    assert_eq!(results.len(), 2, "Should find 2 group chats");

    // Test name filter
    let filter2 = ChatFilter::new().with_name("Work");
    let results2 = manager.list_chats("test", Some(filter2)).await?;
    assert_eq!(results2.len(), 2, "Should find 2 chats with 'Work'");

    // Test combined filters
    let filter3 = ChatFilter::new()
        .with_type(ChatType::Group)
        .with_name("Work");
    let results3 = manager.list_chats("test", Some(filter3)).await?;
    assert_eq!(results3.len(), 2, "Should find 2 work group chats");

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_list_sources() -> Result<()> {
    let manager = SourcesManager::new();

    let source1 = MockChatSource::new("telegram", "Telegram");
    let source2 = MockChatSource::new("signal", "Signal");

    manager.register(Box::new(source1))?;
    manager.register(Box::new(source2))?;

    let request = ListSourcesRequest {};
    let response = handle_list_sources(request, &manager).await?;

    assert_eq!(response.sources.len(), 2);
    assert!(response.sources.iter().any(|s| s.id == "telegram"));
    assert!(response.sources.iter().any(|s| s.id == "signal"));

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_list_chats() -> Result<()> {
    let manager = SourcesManager::new();

    let chats = vec![
        create_chat(1, "Work Team", ChatType::Group),
        create_chat(2, "Personal", ChatType::DirectMessage),
    ];

    let source = MockChatSource::new("telegram", "Telegram").with_chats(chats);
    manager.register(Box::new(source))?;

    let request = ListChatsRequest {
        source: "telegram".to_string(),
        name_pattern: Some("Work".to_string()),
        chat_type: None,
    };

    let response = handle_list_chats(request, &manager).await?;

    assert_eq!(response.chats.len(), 1);
    assert_eq!(response.chats[0].title, Some("Work Team".to_string()));

    Ok(())
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_get_messages() -> Result<()> {
    let manager = SourcesManager::new();

    let now = Utc::now();

    let chats = vec![create_chat(1, "Test Chat", ChatType::Group)];

    let messages = vec![
        create_message(1, 1, 10, "Alice", "Hello world", now - Duration::hours(1)),
        create_message(2, 1, 11, "Bob", "Hi there", now - Duration::hours(2)),
    ];

    let source = MockChatSource::new("telegram", "Telegram")
        .with_chats(chats)
        .with_messages(messages);

    manager.register(Box::new(source))?;

    let request = GetMessagesRequest {
        source: Some("telegram".to_string()),
        chat: "Test".to_string(),
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: Some(10),
    };

    let response = handle_get_messages(request, &manager).await?;

    assert_eq!(response.total, 2);
    assert_eq!(response.messages.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_error_handling_source_not_found() -> Result<()> {
    let manager = SourcesManager::new();

    let filter = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let result = manager.query_messages(Some("nonexistent"), filter).await;
    assert!(result.is_err(), "Should error on nonexistent source");

    Ok(())
}

#[tokio::test]
async fn test_empty_results() -> Result<()> {
    let manager = SourcesManager::new();

    let source = MockChatSource::new("test", "Test");
    manager.register(Box::new(source))?;

    // Query messages from empty source
    let filter = MessageFilter {
        chat: ChatPattern::All,
        since: None,
        before: None,
        sender: None,
        search: None,
        limit: None,
        content_type: None,
    };

    let messages = manager.query_messages(Some("test"), filter).await?;
    assert_eq!(messages.len(), 0, "Should return empty results");

    // List chats from empty source
    let chats = manager.list_chats("test", None).await?;
    assert_eq!(chats.len(), 0, "Should return empty chat list");

    Ok(())
}
