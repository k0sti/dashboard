# WhatsApp Integration - Vertical Slice Quickstart

⚠️ **IMPORTANT WARNING**: This uses an unofficial WhatsApp client library which **may violate WhatsApp/Meta's Terms of Service**. Using this code may result in temporary or permanent account suspension. **Use at your own risk** and only for personal/testing purposes.

## Current Status: VERTICAL SLICE (Scaffold Only)

This is a **minimal vertical slice** implementation focused on getting messages from a single WhatsApp group. The scaffold is in place, but the actual WhatsApp client integration is **NOT YET IMPLEMENTED**.

### ✅ What's Complete:

1. **Dependency setup** - `whatsapp-rust` and `qr2term` added to Cargo.toml
2. **Feature flag** - Build with `--features whatsapp`
3. **Type definitions** - WhatsAppSource implements ChatSource trait
4. **Unified API integration** - Works with existing CLI commands
5. **Configuration** - WhatsAppConfig for session management

### ❌ What Still Needs Implementation:

1. **QR Code Authentication** - `authenticate_with_qr()` function
2. **Session Management** - `load_session()` and save functions
3. **Group Search** - `find_group_by_name()` logic
4. **Message Fetching** - Actual API calls to retrieve messages
5. **Message Conversion** - Map WhatsApp messages to unified type
6. **Error Handling** - Proper WhatsApp-specific errors

## Quick Start (When Implemented)

### Step 1: Build with WhatsApp Support

```bash
cd crates/chat
cargo build --features whatsapp
```

### Step 2: Initialize WhatsApp Connection

```bash
# First time - will show QR code
chat whatsapp init

# Scan QR code with WhatsApp mobile app
# Session will be saved for future use
```

### Step 3: List Your Groups

```bash
# List all WhatsApp groups
chat chats whatsapp --type=group
```

### Step 4: Get Messages from Your Group

```bash
# Get last 100 messages from a specific group
chat messages whatsapp:"Your Group Name" --limit 100

# Export all messages to JSON
chat messages whatsapp:"Your Group Name" --format json > messages.json

# Get messages from last 7 days
chat messages whatsapp:"Your Group Name" --since 7d

# Search for specific text
chat messages whatsapp:"Your Group Name" --search "meeting"
```

## Implementation TODO List

To complete this vertical slice, you need to implement:

### 1. Add WhatsApp Client Initialization (30 min - 1 hour)

```rust
// In whatsapp_source.rs:authenticate_with_qr()

use whatsapp_rust::Client;
use qr2term::print_qr;

async fn authenticate_with_qr(&mut self) -> Result<()> {
    // 1. Create WhatsApp client
    let client = Client::new()?;

    // 2. Get QR code data
    let qr_data = client.get_qr_code().await?;

    // 3. Display QR code in terminal
    print_qr(&qr_data)?;
    println!("Scan this QR code with WhatsApp mobile app");

    // 4. Wait for authentication
    client.wait_for_login().await?;

    // 5. Save session
    client.save_session(&self.session_path).await?;

    self.client = Some(client);
    Ok(())
}
```

### 2. Implement Group Search (1 hour)

```rust
async fn find_group_by_name(&self, name: &str) -> Result<Option<WaChat>> {
    let client = self.client.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

    let chats = client.get_chats().await?;

    let name_lower = name.to_lowercase();
    Ok(chats.into_iter()
        .filter(|chat| chat.is_group())
        .find(|chat| {
            chat.name()
                .map(|n| n.to_lowercase().contains(&name_lower))
                .unwrap_or(false)
        }))
}
```

### 3. Implement Message Fetching (2-3 hours)

```rust
async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>> {
    let client = self.client.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

    // Extract group name
    let group_name = match &filter.chat {
        ChatPattern::Name(name) => name,
        _ => bail!("Only group name search supported"),
    };

    // Find group
    let group = self.find_group_by_name(group_name).await?
        .ok_or_else(|| anyhow::anyhow!("Group '{}' not found", group_name))?;

    // Fetch messages
    let limit = filter.limit.unwrap_or(100);
    let wa_messages = client.get_messages(&group.id(), limit).await?;

    // Convert to unified format
    let mut messages: Vec<Message> = wa_messages
        .iter()
        .filter_map(|msg| self.convert_message(msg, group.id()).ok())
        .collect();

    // Apply time filters
    if let Some(since) = filter.since {
        messages.retain(|msg| msg.timestamp >= since);
    }
    if let Some(before) = filter.before {
        messages.retain(|msg| msg.timestamp <= before);
    }

    // Apply search filter
    if let Some(ref search_term) = filter.search {
        let search_lower = search_term.to_lowercase();
        messages.retain(|msg| {
            matches!(&msg.content, MessageContent::Text(text)
                if text.to_lowercase().contains(&search_lower))
        });
    }

    Ok(messages)
}
```

### 4. Implement Message Conversion (1 hour)

```rust
fn convert_message(&self, wa_msg: &WaMessage, chat_id: &str) -> Result<Message> {
    Ok(Message {
        id: MessageId::new(wa_msg.id().to_string()),
        chat_id: ChatId::new(chat_id.to_string()),
        sender: User {
            id: UserId::new(wa_msg.sender().to_string()),
            username: None,
            display_name: wa_msg.sender_name().map(|s| s.to_string()),
            phone_number: Some(wa_msg.sender().to_string()),
        },
        content: match wa_msg.content_type() {
            WaContentType::Text => MessageContent::Text(
                wa_msg.text().unwrap_or("").to_string()
            ),
            WaContentType::Image => MessageContent::Image {
                caption: wa_msg.caption().map(|s| s.to_string()),
                url: wa_msg.media_url().map(|s| s.to_string()),
            },
            WaContentType::Video => MessageContent::Video {
                caption: wa_msg.caption().map(|s| s.to_string()),
                url: wa_msg.media_url().map(|s| s.to_string()),
            },
            _ => MessageContent::Unknown,
        },
        timestamp: wa_msg.timestamp(),
        reply_to: wa_msg.quoted_message_id().map(|id| MessageId::new(id.to_string())),
        edited: false,
    })
}
```

## Estimated Implementation Time

- **Minimal (group messages only)**: 4-6 hours
- **Full WhatsApp CLI**: 2-3 weeks (see OpenSpec tasks)

## Why Vertical Slice?

A vertical slice means implementing **only what you need** to get one use case working end-to-end:

✅ **You Get:**
- QR code authentication
- Read messages from ONE group
- All existing unified API features (filtering, output formats, etc.)
- CLI commands already work

❌ **You Skip:**
- Direct messages
- Media download
- Watch mode (real-time streaming)
- Multiple groups at once
- Send messages (read-only)

## Testing Your Implementation

1. **Unit Tests**: Already scaffolded in `whatsapp_source.rs`

2. **Integration Test**:
```bash
# Test authentication
cargo run --features whatsapp -- whatsapp init

# Test message fetching
cargo run --features whatsapp -- messages whatsapp:"Test Group" --limit 10
```

3. **Use Test Account**: **DO NOT** use your main WhatsApp account during development!

## Troubleshooting

### Error: "WhatsApp support not enabled"
**Solution**: Build with `--features whatsapp`

### Error: "Not connected to WhatsApp"
**Solution**: Run `chat whatsapp init` first

### Error: "Session expired"
**Solution**: Delete session file and re-authenticate with QR code

### Account Suspended
**This is the risk of using unofficial clients.** WhatsApp/Meta can detect and ban accounts using third-party clients.

## Next Steps

1. **Review whatsapp-rust documentation**: https://github.com/jlucaso1/whatsapp-rust
2. **Implement the 4 TODO functions** above
3. **Test with a test WhatsApp account** (not your main account!)
4. **Expand gradually** if needed

## Complete OpenSpec Tasks

For a full WhatsApp CLI implementation, see:
- `openspec/changes/add-whatsapp-cli/tasks.md` (61 tasks)
- `openspec/changes/add-whatsapp-cli/proposal.md`

This vertical slice covers approximately **20% of the full OpenSpec** but gives you **100% of what you need** for your specific use case.
