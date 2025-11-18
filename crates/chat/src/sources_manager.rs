use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::types::{ChatFilter, ChatSource, Message, MessageFilter, SourceInfo};

/// Manager for multiple chat sources
pub struct SourcesManager {
    sources: Arc<RwLock<HashMap<String, Box<dyn ChatSource>>>>,
}

impl SourcesManager {
    /// Create a new empty sources manager
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new source
    pub fn register(&self, source: Box<dyn ChatSource>) -> Result<()> {
        let source_id = source.source_id().to_string();

        let mut sources = self.sources.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        if sources.contains_key(&source_id) {
            anyhow::bail!("Source '{}' is already registered", source_id);
        }

        sources.insert(source_id, source);
        Ok(())
    }

    /// Unregister a source by ID
    pub fn unregister(&self, source_id: &str) -> Result<()> {
        let mut sources = self.sources.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;

        if sources.remove(source_id).is_none() {
            anyhow::bail!("Source '{}' not found", source_id);
        }

        Ok(())
    }

    /// Get a source by ID
    /// Note: This returns None rather than a reference to avoid lifetime issues
    /// For operations on a source, use the query methods instead
    pub fn has_source(&self, source_id: &str) -> bool {
        self.sources.read()
            .map(|sources| sources.contains_key(source_id))
            .unwrap_or(false)
    }

    /// List all registered sources
    pub fn list_sources(&self) -> Result<Vec<SourceInfo>> {
        let sources = self.sources.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        let mut source_infos: Vec<SourceInfo> = sources.values()
            .map(|source| SourceInfo {
                id: source.source_id().to_string(),
                name: source.source_name().to_string(),
                is_connected: source.is_connected(),
            })
            .collect();

        // Sort by ID for consistent ordering
        source_infos.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(source_infos)
    }

    /// List chats from a specific source
    pub async fn list_chats(&self, source_id: &str, filter: Option<ChatFilter>) -> Result<Vec<crate::types::Chat>> {
        let sources = self.sources.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

        let source = sources.get(source_id)
            .ok_or_else(|| anyhow::anyhow!("Source '{}' not found", source_id))?;

        if !source.is_connected() {
            anyhow::bail!("Source '{}' is not connected", source_id);
        }

        source.list_chats(filter).await
    }

    /// Query messages from a specific source
    pub async fn query_messages(&self, source_id: Option<&str>, filter: MessageFilter) -> Result<Vec<Message>> {
        filter.validate()?;

        if let Some(id) = source_id {
            // Query specific source
            let sources = self.sources.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

            let source = sources.get(id)
                .ok_or_else(|| anyhow::anyhow!("Source '{}' not found", id))?;

            if !source.is_connected() {
                anyhow::bail!("Source '{}' is not connected", id);
            }

            source.get_messages(filter).await
        } else {
            // Query all sources
            let source_ids: Vec<String> = {
                let sources = self.sources.read()
                    .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;
                sources.keys().cloned().collect()
            };

            let mut all_messages = Vec::new();

            for id in source_ids {
                let sources = self.sources.read()
                    .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;

                if let Some(source) = sources.get(&id) {
                    if source.is_connected() {
                        match source.get_messages(filter.clone()).await {
                            Ok(mut messages) => all_messages.append(&mut messages),
                            Err(e) => {
                                eprintln!("Warning: Failed to query source '{}': {}", id, e);
                            }
                        }
                    }
                }
            }

            // Sort by timestamp (most recent first)
            all_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            // Apply limit if specified
            if let Some(limit) = filter.limit {
                all_messages.truncate(limit);
            }

            Ok(all_messages)
        }
    }

    /// Get number of registered sources
    pub fn count(&self) -> usize {
        self.sources.read()
            .map(|sources| sources.len())
            .unwrap_or(0)
    }

    /// Check if any sources are registered
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

impl Default for SourcesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SourcesManager {
    fn clone(&self) -> Self {
        Self {
            sources: Arc::clone(&self.sources),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Chat, ChatId, ChatType};
    use async_trait::async_trait;

    // Mock source for testing
    struct MockSource {
        id: String,
        name: String,
        connected: bool,
    }

    #[async_trait]
    impl ChatSource for MockSource {
        fn source_id(&self) -> &str {
            &self.id
        }

        fn source_name(&self) -> &str {
            &self.name
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        async fn list_chats(&self, _filter: Option<ChatFilter>) -> Result<Vec<Chat>> {
            Ok(vec![Chat {
                id: ChatId::new("test-chat"),
                title: Some("Test Chat".to_string()),
                chat_type: ChatType::DirectMessage,
                participant_count: Some(2),
            }])
        }

        async fn get_messages(&self, _filter: MessageFilter) -> Result<Vec<Message>> {
            Ok(vec![])
        }

        async fn subscribe(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>> {
            Ok(None)
        }
    }

    #[test]
    fn test_new_manager() {
        let manager = SourcesManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_register_source() {
        let manager = SourcesManager::new();
        let source = Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        });

        let result = manager.register(source);
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
        assert!(manager.has_source("test"));
    }

    #[test]
    fn test_register_duplicate() {
        let manager = SourcesManager::new();
        let source1 = Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        });
        let source2 = Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        });

        manager.register(source1).unwrap();
        let result = manager.register(source2);
        assert!(result.is_err());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_unregister() {
        let manager = SourcesManager::new();
        let source = Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        });

        manager.register(source).unwrap();
        assert_eq!(manager.count(), 1);

        let result = manager.unregister("test");
        assert!(result.is_ok());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_list_sources() {
        let manager = SourcesManager::new();

        manager.register(Box::new(MockSource {
            id: "telegram".to_string(),
            name: "Telegram".to_string(),
            connected: true,
        })).unwrap();

        manager.register(Box::new(MockSource {
            id: "signal".to_string(),
            name: "Signal".to_string(),
            connected: false,
        })).unwrap();

        let sources = manager.list_sources().unwrap();
        assert_eq!(sources.len(), 2);

        // Should be sorted by ID
        assert_eq!(sources[0].id, "signal");
        assert_eq!(sources[1].id, "telegram");

        assert!(sources[1].is_connected);
        assert!(!sources[0].is_connected);
    }

    #[tokio::test]
    async fn test_list_chats() {
        let manager = SourcesManager::new();

        manager.register(Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        })).unwrap();

        let chats = manager.list_chats("test", None).await.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].title.as_ref().unwrap(), "Test Chat");
    }

    #[tokio::test]
    async fn test_list_chats_disconnected() {
        let manager = SourcesManager::new();

        manager.register(Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: false,
        })).unwrap();

        let result = manager.list_chats("test", None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_clone() {
        let manager1 = SourcesManager::new();
        manager1.register(Box::new(MockSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            connected: true,
        })).unwrap();

        let manager2 = manager1.clone();
        assert_eq!(manager2.count(), 1);
        assert!(manager2.has_source("test"));
    }
}
