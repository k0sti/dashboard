# Crates

This directory contains reusable library crates that are part of the agent dashboard workspace.

## Chat (`chat/`)

Generic interface for reading messages from multiple chat platforms (Telegram, WhatsApp, Signal).

**Features:**
- Platform-agnostic trait-based design
- Support for Matrix-based unified access via mautrix bridges
- Direct API implementations for Telegram (and other platforms)
- Read-only message access
- Async/streaming support

**Usage:**
```rust
use chat::{ChatClient, TelegramChatClient, ChatClientConfig};

// Create and use a chat client
let client = TelegramChatClient::new(config)?;
client.connect().await?;
let messages = client.get_messages(&chat_id, options).await?;
```

See `chat/README.md` and `chat/IMPLEMENTATION.md` for details.

## Adding New Crates

To add a new crate to this workspace:

1. Create directory: `crates/my-crate/`
2. Initialize: `cargo init --lib crates/my-crate`
3. Add to main `Cargo.toml`:
   ```toml
   [dependencies]
   my-crate = { path = "crates/my-crate" }
   ```
4. Use in main project: Import and use as any external crate

## Why Separate Crates?

Benefits of this structure:
- **Modularity**: Each crate has clear boundaries and responsibilities
- **Reusability**: Crates can be used independently or in other projects
- **Testing**: Easier to test crates in isolation
- **Compilation**: Parallel compilation and incremental builds
- **Publishing**: Can publish crates to crates.io independently
