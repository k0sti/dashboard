# Chat Client Interface

A generic, read-only interface for accessing messages from multiple chat platforms (Telegram, WhatsApp, Signal).

## Architecture

This module provides a platform-agnostic trait `ChatClient` that can be implemented for different messaging platforms. The design focuses on **read-only** access to messages, making it suitable for monitoring, archiving, or building dashboards.

## Core Types

### `ChatClient` Trait

The main trait that all chat clients must implement:

```rust
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    fn get_config(&self) -> &ChatClientConfig;
    fn get_status(&self) -> ChatClientStatus;
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn list_chats(&self) -> Result<Vec<Chat>>;
    async fn get_messages(&self, chat_id: &ChatId, options: MessageFetchOptions) -> Result<Vec<Message>>;
    async fn get_message(&self, chat_id: &ChatId, message_id: &MessageId) -> Result<Option<Message>>;
    async fn subscribe_messages(&self) -> Result<Option<tokio::sync::mpsc::Receiver<Message>>>;
}
```

### Key Types

- `Message`: Represents a single message with content, sender, timestamp, etc.
- `Chat`: Represents a conversation/chat room
- `User`: Represents a user/sender
- `MessageContent`: Enum for different message types (text, image, video, etc.)
- `MessageFetchOptions`: Options for filtering and paginating messages

## Implementation Approaches

There are two main approaches to implementing chat clients:

### Approach 1: Matrix-based (Unified)

Use the Matrix protocol with mautrix bridges to access all platforms through a single API.

**Pros:**
- Single implementation for all platforms
- Well-maintained bridges
- Consistent API across platforms
- Official Rust SDK available

**Cons:**
- Requires running a Matrix homeserver
- Requires setting up and configuring bridges for each platform
- Additional infrastructure complexity

**Dependencies:**
```toml
matrix-sdk = "0.14"
```

**Implementation outline:**
```rust
pub struct MatrixChatClient {
    client: matrix_sdk::Client,
    config: ChatClientConfig,
    // ...
}

impl ChatClient for MatrixChatClient {
    // Implement using matrix-sdk
    // Messages from Telegram/WhatsApp/Signal appear as Matrix messages
}
```

### Approach 2: Direct Platform APIs

Implement separate clients for each platform using their native APIs.

#### Telegram

**Library:** `grammers` or `tdlib-rs`

**Pros:**
- Direct access to Telegram API
- No intermediary services needed
- Full feature support

**Dependencies:**
```toml
grammers-client = "0.6"
grammers-session = "0.6"
```

**Implementation outline:**
```rust
pub struct TelegramChatClient {
    client: grammers_client::Client,
    config: ChatClientConfig,
    // ...
}
```

#### WhatsApp

**Library:** `whatsapp-rust` (inspired by whatsmeow)

**⚠️ Warning:** Using unofficial WhatsApp clients may violate Terms of Service

**Dependencies:**
```toml
# whatsapp-rust is still in development
# Check: https://github.com/jlucaso1/whatsapp-rust
```

**Implementation outline:**
```rust
pub struct WhatsAppChatClient {
    // Implementation using whatsapp-rust
    config: ChatClientConfig,
    // ...
}
```

#### Signal

**Library:** `libsignal` (official, but use outside Signal is unsupported)

**⚠️ Note:** Signal's libsignal is designed for Signal's own use. Community alternatives may be needed.

**Implementation outline:**
```rust
pub struct SignalChatClient {
    // Implementation details
    config: ChatClientConfig,
    // ...
}
```

## Recommended Approach

For a **minimal implementation** focused on reading messages:

1. **Start with Matrix + mautrix bridges** (Approach 1)
   - Provides unified access to all three platforms
   - Well-tested and maintained
   - Single codebase

2. **Alternative: Telegram-only** (Approach 2)
   - Easiest to implement directly
   - Best documentation and libraries
   - No ToS concerns

## Example Usage

```rust
use chat::{ChatClient, MessageFetchOptions};

async fn read_messages(client: &impl ChatClient) -> anyhow::Result<()> {
    // Connect to the platform
    client.connect().await?;

    // List all chats
    let chats = client.list_chats().await?;

    for chat in chats {
        println!("Chat: {}", chat.title.unwrap_or_else(|| "Untitled".to_string()));

        // Get recent messages
        let options = MessageFetchOptions {
            limit: Some(10),
            ..Default::default()
        };

        let messages = client.get_messages(&chat.id, options).await?;

        for msg in messages {
            if let MessageContent::Text(text) = msg.content {
                println!("  [{}] {}: {}",
                    msg.timestamp,
                    msg.sender.username.unwrap_or_else(|| "Unknown".to_string()),
                    text
                );
            }
        }
    }

    Ok(())
}
```

## Configuration

Each chat client requires platform-specific configuration stored in `ChatClientConfig.config_data`:

### Matrix Configuration
```json
{
  "homeserver_url": "https://matrix.example.com",
  "username": "user",
  "password": "pass"
}
```

### Telegram Configuration
```json
{
  "api_id": "12345",
  "api_hash": "abcdef123456",
  "phone": "+1234567890"
}
```

## Implementation Checklist

- [x] Define core types and traits
- [ ] Implement Matrix-based client
- [ ] Implement Telegram direct client
- [ ] Add authentication handling
- [ ] Add message caching
- [ ] Add error recovery
- [ ] Add rate limiting
- [ ] Add integration tests

## Security Considerations

1. **Credentials**: Store API keys and passwords securely (use system keyring)
2. **Rate Limiting**: Implement proper rate limiting to avoid bans
3. **ToS Compliance**: Be aware of each platform's Terms of Service
4. **Data Privacy**: Handle user data responsibly

## Resources

- **Matrix SDK**: https://github.com/matrix-org/matrix-rust-sdk
- **mautrix Bridges**: https://github.com/mautrix
- **Grammers (Telegram)**: https://github.com/Lonami/grammers
- **TDLib Rust**: https://github.com/paper-plane-developers/tdlib-rs
- **WhatsApp Rust**: https://github.com/jlucaso1/whatsapp-rust
