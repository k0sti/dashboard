# Chat Client Implementation Guide

This document provides step-by-step instructions for implementing the chat client interface for different platforms.

## Quick Start

The chat module provides a generic trait-based interface for reading messages from Telegram, WhatsApp, and Signal. Two stub implementations are provided:

1. **MatrixChatClient** - Unified access via Matrix bridges
2. **TelegramChatClient** - Direct Telegram API access

## Adding Dependencies

### For Matrix Implementation

Add to `Cargo.toml`:

```toml
[dependencies]
matrix-sdk = "0.14"
```

Then uncomment the implementation code in `src/chat/matrix_client.rs`.

### For Telegram Implementation

Add to `Cargo.toml`:

```toml
[dependencies]
grammers-client = "0.6"
grammers-session = "0.6"
grammers-tl-types = "0.6"
```

Then uncomment the implementation code in `src/chat/telegram_client.rs`.

### For WhatsApp Implementation (Future)

Currently, `whatsapp-rust` is in development. Monitor:
- https://github.com/jlucaso1/whatsapp-rust

Once stable, add:

```toml
[dependencies]
# whatsapp-rust = "0.1"  # When available
```

## Implementation Steps

### Step 1: Set Up Infrastructure (Matrix Approach)

If using the Matrix approach for unified access:

1. **Install and configure a Matrix homeserver** (e.g., Synapse)
   ```bash
   # Example using Docker
   docker run -d --name synapse \
     -v synapse-data:/data \
     -p 8008:8008 \
     matrixdotorg/synapse:latest
   ```

2. **Install and configure mautrix bridges**
   - Telegram: https://github.com/mautrix/telegram
   - WhatsApp: https://github.com/mautrix/whatsapp
   - Signal: https://github.com/mautrix/signal

3. **Create a Matrix account** for your bot/client

### Step 2: Implement Connection Logic

In your chosen client implementation:

1. Add the SDK dependency
2. Uncomment the connect() method implementation
3. Handle authentication (may require interactive code input for Telegram)
4. Implement session persistence

Example for Telegram:
```rust
// Handle first-time authentication
if !client.is_authorized().await? {
    let token = client.request_login_code(&phone).await?;

    // Get code from user (could be via UI, stdin, etc.)
    let code = get_auth_code_from_user()?;

    client.sign_in(&token, &code).await?;
    client.session().save_to_file("session.dat")?;
}
```

### Step 3: Implement Message Fetching

Uncomment and complete the implementations of:
- `list_chats()` - Enumerate all chats/rooms
- `get_messages()` - Fetch messages with pagination
- `get_message()` - Fetch a specific message

### Step 4: Implement Message Streaming (Optional)

For real-time updates, implement `subscribe_messages()`:

```rust
async fn subscribe_messages(&self) -> Result<Option<Receiver<Message>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Set up event handler
    // Forward new messages to channel

    Ok(Some(rx))
}
```

### Step 5: Add Configuration Management

Create a configuration file or UI for managing:
- API credentials
- Session files
- Homeserver URLs
- Bridge settings

Example configuration structure:
```json
{
  "clients": [
    {
      "id": "uuid-here",
      "name": "My Telegram",
      "platform": "Telegram",
      "config_data": {
        "api_id": 12345,
        "api_hash": "abc...",
        "phone": "+1234567890",
        "session_file": "telegram.session"
      }
    }
  ]
}
```

## Usage Example

```rust
use anyhow::Result;
use chat::{ChatClient, TelegramChatClient, ChatClientConfig, ChatPlatform};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client configuration
    let config = ChatClientConfig {
        id: ChatClientId::new(),
        name: "My Telegram".to_string(),
        platform: ChatPlatform::Telegram,
        config_data: serde_json::json!({
            "api_id": 12345,
            "api_hash": "your_api_hash",
            "phone": "+1234567890",
            "session_file": "session.dat"
        }),
    };

    // Create and connect client
    let mut client = TelegramChatClient::new(config)?;
    client.connect().await?;

    // List all chats
    let chats = client.list_chats().await?;
    println!("Found {} chats", chats.len());

    // Read messages from first chat
    if let Some(chat) = chats.first() {
        let messages = client.get_messages(
            &chat.id,
            MessageFetchOptions {
                limit: Some(10),
                ..Default::default()
            }
        ).await?;

        for msg in messages {
            if let MessageContent::Text(text) = msg.content {
                println!("[{}] {}: {}",
                    msg.timestamp.format("%Y-%m-%d %H:%M"),
                    msg.sender.username.unwrap_or("Unknown".to_string()),
                    text
                );
            }
        }
    }

    Ok(())
}
```

## Testing

1. **Unit Tests**: Test individual message parsing functions
2. **Integration Tests**: Test against test accounts
3. **Manual Testing**: Verify with real chats

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ChatClientConfig {
            id: ChatClientId::new(),
            name: "Test".to_string(),
            platform: ChatPlatform::Telegram,
            config_data: serde_json::json!({
                "api_id": 12345,
                "api_hash": "test",
                "phone": "+1234567890"
            }),
        };

        let client = TelegramChatClient::new(config);
        assert!(client.is_ok());
    }
}
```

## Security Considerations

1. **Credentials Storage**
   - Never commit credentials to version control
   - Use environment variables or secure key storage
   - Consider using system keyring (e.g., `keyring` crate)

2. **Rate Limiting**
   - Implement exponential backoff
   - Respect platform rate limits
   - Cache frequently accessed data

3. **Session Management**
   - Encrypt session files at rest
   - Implement session expiration
   - Handle re-authentication gracefully

4. **Data Privacy**
   - Don't log message contents
   - Implement data retention policies
   - Follow GDPR/privacy regulations

## Troubleshooting

### Connection Issues

- Check firewall settings
- Verify API credentials
- Check homeserver/bridge status (for Matrix)
- Review authentication flow

### Message Fetching Issues

- Verify chat permissions
- Check pagination parameters
- Handle rate limits

### Authentication Issues

- Ensure phone number format is correct
- Check API credentials validity
- Verify session file permissions

## Next Steps

1. Choose an implementation approach (Matrix vs Direct)
2. Set up required infrastructure
3. Add dependencies to Cargo.toml
4. Implement authentication flow
5. Test with a single chat
6. Expand to multiple chats
7. Add error handling and retry logic
8. Implement message caching
9. Add UI integration (if needed)

## Resources

- [Matrix Rust SDK Docs](https://docs.rs/matrix-sdk/)
- [Grammers Docs](https://docs.rs/grammers-client/)
- [mautrix Bridge Docs](https://docs.mau.fi/)
- [Telegram API Docs](https://core.telegram.org/api)

## Contributing

When implementing:
1. Follow the existing code style
2. Add comprehensive error handling
3. Document all public APIs
4. Add tests for new functionality
5. Update this guide with lessons learned
