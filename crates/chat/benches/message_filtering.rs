use chat::{
    types::*, ChatSource, MessageFilter, SourcesManager, ChatPattern,
};
use chrono::{DateTime, Utc, Duration};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration as StdDuration;

/// Mock chat source for benchmarking
struct BenchmarkChatSource {
    id: String,
    name: String,
    chats: Vec<Chat>,
    messages: Vec<Message>,
}

impl BenchmarkChatSource {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            chats: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    fn with_chats(mut self, chats: Vec<Chat>) -> Self {
        self.chats = chats;
        self
    }
}

#[async_trait::async_trait]
impl ChatSource for BenchmarkChatSource {
    fn source_id(&self) -> &str {
        &self.id
    }

    fn source_name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        true
    }

    async fn list_chats(&self, _filter: Option<ChatFilter>) -> anyhow::Result<Vec<Chat>> {
        Ok(self.chats.clone())
    }

    async fn get_messages(&self, filter: MessageFilter) -> anyhow::Result<Vec<Message>> {
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
                // Not implemented for benchmark
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

    async fn subscribe(&self) -> anyhow::Result<Option<tokio::sync::mpsc::Receiver<Message>>> {
        Ok(None)
    }
}

/// Generate test messages
fn generate_messages(count: usize) -> Vec<Message> {
    let base_time = Utc::now() - Duration::days(30);
    let mut messages = Vec::with_capacity(count);

    for i in 0..count {
        let message = Message {
            id: MessageId::new(i.to_string()),
            chat_id: ChatId::new((i % 10).to_string()),
            sender: User {
                id: UserId::new((i % 5).to_string()),
                username: Some(format!("user_{}", i % 5)),
                display_name: Some(format!("User {}", i % 5)),
                phone_number: None,
            },
            content: MessageContent::Text(format!(
                "Test message {} with some content to search through",
                i
            )),
            timestamp: base_time + Duration::seconds(i as i64),
            reply_to: None,
            edited: i % 10 == 0,
        };
        messages.push(message);
    }

    messages
}

/// Generate test chats
fn generate_chats(count: usize) -> Vec<Chat> {
    let mut chats = Vec::with_capacity(count);

    for i in 0..count {
        let chat = Chat {
            id: ChatId::new(i.to_string()),
            title: Some(format!("Chat {}", i)),
            chat_type: if i % 3 == 0 {
                ChatType::DirectMessage
            } else if i % 3 == 1 {
                ChatType::Group
            } else {
                ChatType::Channel
            },
            participant_count: Some((i % 50) + 2),
        };
        chats.push(chat);
    }

    chats
}

fn benchmark_query_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_all_messages");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages = generate_messages(size);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();
            let source = BenchmarkChatSource::new("test", "Test")
                .with_messages(messages)
                .with_chats(chats);

            manager.register(Box::new(source)).unwrap();

            let filter = MessageFilter {
                chat: ChatPattern::All,
                since: None,
                before: None,
                sender: None,
                search: None,
                limit: None,
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager
                    .query_messages(Some("test"), filter.clone())
                    .await
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn benchmark_time_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_filtered_messages");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages = generate_messages(size);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();
            let source = BenchmarkChatSource::new("test", "Test")
                .with_messages(messages)
                .with_chats(chats);

            manager.register(Box::new(source)).unwrap();

            // Filter for last 7 days
            let filter = MessageFilter {
                chat: ChatPattern::All,
                since: Some(Utc::now() - Duration::days(7)),
                before: None,
                sender: None,
                search: None,
                limit: None,
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager
                    .query_messages(Some("test"), filter.clone())
                    .await
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn benchmark_search_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_filtered_messages");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages = generate_messages(size);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();
            let source = BenchmarkChatSource::new("test", "Test")
                .with_messages(messages)
                .with_chats(chats);

            manager.register(Box::new(source)).unwrap();

            // Search for "message"
            let filter = MessageFilter {
                chat: ChatPattern::All,
                since: None,
                before: None,
                sender: None,
                search: Some("message".to_string()),
                limit: None,
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager
                    .query_messages(Some("test"), filter.clone())
                    .await
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn benchmark_combined_filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_filtered_messages");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages = generate_messages(size);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();
            let source = BenchmarkChatSource::new("test", "Test")
                .with_messages(messages)
                .with_chats(chats);

            manager.register(Box::new(source)).unwrap();

            // Combined: time + search + sender + limit
            let filter = MessageFilter {
                chat: ChatPattern::All,
                since: Some(Utc::now() - Duration::days(7)),
                before: None,
                sender: Some("User 1".to_string()),
                search: Some("message".to_string()),
                limit: Some(100),
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager
                    .query_messages(Some("test"), filter.clone())
                    .await
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn benchmark_chat_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("chat_pattern_matching");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages = generate_messages(size);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();
            let source = BenchmarkChatSource::new("test", "Test")
                .with_messages(messages)
                .with_chats(chats);

            manager.register(Box::new(source)).unwrap();

            // Match by chat name
            let filter = MessageFilter {
                chat: ChatPattern::Name("Chat 5".to_string()),
                since: None,
                before: None,
                sender: None,
                search: None,
                limit: None,
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager
                    .query_messages(Some("test"), filter.clone())
                    .await
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn benchmark_cross_source_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_source_queries");
    group.measurement_time(StdDuration::from_secs(10));

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let messages1 = generate_messages(size / 2);
            let messages2 = generate_messages(size / 2);
            let chats = generate_chats(size / 10);

            let manager = SourcesManager::new();

            let source1 = BenchmarkChatSource::new("source1", "Source 1")
                .with_messages(messages1)
                .with_chats(chats.clone());

            let source2 = BenchmarkChatSource::new("source2", "Source 2")
                .with_messages(messages2)
                .with_chats(chats);

            manager.register(Box::new(source1)).unwrap();
            manager.register(Box::new(source2)).unwrap();

            // Query all sources
            let filter = MessageFilter {
                chat: ChatPattern::All,
                since: None,
                before: None,
                sender: None,
                search: None,
                limit: None,
                content_type: None,
            };

            b.to_async(&runtime).iter(|| async {
                let results = manager.query_messages(None, filter.clone()).await.unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_query_all,
    benchmark_time_filter,
    benchmark_search_filter,
    benchmark_combined_filters,
    benchmark_chat_pattern_matching,
    benchmark_cross_source_queries,
);
criterion_main!(benches);
