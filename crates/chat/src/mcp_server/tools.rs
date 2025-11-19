use anyhow::Result;

use crate::SourcesManager;

use super::*;

/// Handle list_sources tool call
pub async fn handle_list_sources(
    _request: ListSourcesRequest,
    manager: &SourcesManager,
) -> Result<ListSourcesResponse> {
    let sources = manager.list_sources()?;
    let sources: Vec<SourceInfo> = sources.into_iter().map(|s| s.into()).collect();

    Ok(ListSourcesResponse { sources })
}

/// Handle list_chats tool call
pub async fn handle_list_chats(
    request: ListChatsRequest,
    manager: &SourcesManager,
) -> Result<ListChatsResponse> {
    // Check if source exists
    if !manager.has_source(&request.source) {
        anyhow::bail!("Source '{}' not found. Available sources can be listed with list_sources tool.", request.source);
    }

    // Build filter
    let filter = build_chat_filter(&request)?;

    // List chats
    let chats = manager.list_chats(&request.source, filter).await?;
    let chats: Vec<ChatInfo> = chats.iter().map(|c| c.into()).collect();

    Ok(ListChatsResponse { chats })
}

/// Handle get_messages tool call
pub async fn handle_get_messages(
    request: GetMessagesRequest,
    manager: &SourcesManager,
) -> Result<GetMessagesResponse> {
    // Build message filter
    let filter = build_message_filter(&request).await?;

    // Query messages
    let messages = manager
        .query_messages(request.source.as_deref(), filter)
        .await?;

    let total = messages.len();
    let messages: Vec<MessageInfo> = messages.iter().map(|m| m.into()).collect();

    Ok(GetMessagesResponse { messages, total })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_list_sources_empty() {
        let manager = SourcesManager::new();
        let request = ListSourcesRequest {};

        let response = handle_list_sources(request, &manager).await.unwrap();
        assert_eq!(response.sources.len(), 0);
    }

    #[tokio::test]
    async fn test_handle_list_chats_source_not_found() {
        let manager = SourcesManager::new();
        let request = ListChatsRequest {
            source: "nonexistent".to_string(),
            name_pattern: None,
            chat_type: None,
        };

        let result = handle_list_chats(request, &manager).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
