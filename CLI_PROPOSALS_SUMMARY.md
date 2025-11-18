# Chat CLI Binary Proposals Summary

Three OpenSpec proposals have been created for command-line interfaces to read messages from Telegram, WhatsApp, and Signal.

## Proposals Created

1. **add-telegram-cli** - Telegram CLI binary (52 tasks)
2. **add-whatsapp-cli** - WhatsApp CLI binary (61 tasks)
3. **add-signal-cli** - Signal CLI binary (67 tasks)

All proposals have been validated with `openspec validate --strict` and are ready for review.

## Common Command Structure

All three CLIs follow the same command structure:

```bash
chat <platform> <command> [options]
```

Where `<platform>` is one of: `telegram`, `whatsapp`, or `signal`

## Core Commands

### 1. `init` - Initialize and authenticate
- Telegram: Phone number + verification code
- WhatsApp: QR code scanning
- Signal: Phone registration or device linking via QR code

```bash
chat telegram init
chat whatsapp init
chat signal init
chat signal init --link  # Link as secondary device
```

### 2. `status` - Check connection status
```bash
chat telegram status
chat whatsapp status
chat signal status
```

### 3. `list` - List all chats/groups
```bash
chat telegram list
chat telegram list --format json
chat telegram list --type group
```

### 4. `get` - Retrieve messages from a chat
```bash
chat telegram get "Chat Name"
chat telegram get --id 123456789
chat whatsapp get "+1234567890"
chat signal get --uuid "550e8400-e29b-41d4-a716-446655440000"
```

### 5. `watch` - Stream new messages in real-time
```bash
chat telegram watch "Chat Name"
chat whatsapp watch --all
chat signal watch "Contact"
```

### 6. `export` - Export messages to file
```bash
chat telegram export "Chat" --format json --output messages.json
chat whatsapp export "Contact" --format csv --output messages.csv
```

## Message Filtering Options

All platforms support the same filtering options:

### Time-based Filtering

```bash
# Absolute timestamp
--since "2025-01-01T00:00:00Z"
--before "2025-01-15T00:00:00Z"

# Relative time
--since "2 days ago"
--since "7 days ago"
--before "2 days ago"

# Time range
chat telegram get "Chat" --since "7 days ago" --before "2 days ago"
```

### Count Limiting

```bash
# Limit number of messages (default: 100)
--limit 50

# Get last 10 messages
chat telegram get "Chat" --limit 10
```

### Sender Filtering (for groups)

```bash
# By username/name
--sender "@username"
--sender "John Doe"

# By ID/UUID
--sender-id 123456
--sender-uuid "550e8400-..."
```

### Message Type Filtering

```bash
# Single type
--type text
--type media
--type image

# Multiple types
--type text,image,video
```

## Output Formats

All CLIs support multiple output formats:

```bash
--format text      # Human-readable (default)
--format json      # JSON array
--format csv       # CSV file
--format compact   # Single-line compact format

# Write to file
--output messages.json
```

## Additional Functionality

Beyond the requested features, the proposals include:

### 1. Configuration Management
```bash
chat telegram config set api_id 12345
chat telegram config get api_id
chat telegram config list
```

### 2. Search Messages
```bash
chat telegram search "Chat" "search term"
chat telegram search --all "search term"
chat telegram search "Chat" "term" --ignore-case
```

### 3. Chat Information
```bash
chat telegram info "Chat Name"
chat whatsapp info "Group Name"
chat signal info "Contact"
```

### 4. Media Download
```bash
# Download specific media
chat whatsapp download --message-id "abc123" --output ./media/

# Download all media from chat
chat signal download "Contact" --all --output ./media/

# Download with filters
chat telegram download "Chat" --type image --since "7 days ago"
```

### 5. Signal-Specific: Safety Number Verification
```bash
chat signal verify "Contact"
chat signal verify "Contact" --trust
```

### 6. Session Management
```bash
# Clear session/logout
chat telegram logout
chat whatsapp logout
chat signal logout
```

## Platform-Specific Considerations

### Telegram
- Uses API ID and API hash from Telegram's developer portal
- Session stored in file for persistence
- Supports channels, groups, and DMs
- Most stable and documented API

### WhatsApp
- Uses QR code for authentication (WhatsApp Web protocol)
- **Warning:** Unofficial client - may violate Terms of Service
- Supports individual chats and groups
- Chat IDs: `@c.us` (individual), `@g.us` (group)

### Signal
- Can register new number OR link as secondary device
- Uses UUID-based addressing
- Supports safety number verification
- Handles disappearing messages
- Most privacy-focused

## Implementation Architecture

Each CLI will be structured as:

```
crates/
├── chat/               # Shared chat interface library
├── chat-telegram/      # Telegram CLI binary
├── chat-whatsapp/      # WhatsApp CLI binary
└── chat-signal/        # Signal CLI binary
```

Each binary crate will:
1. Use `clap` for command-line parsing
2. Depend on the `chat` crate for common types
3. Implement platform-specific authentication
4. Support all common filtering and output options
5. Provide comprehensive `--help` documentation

## Usage Examples

### Example 1: Monitor recent messages
```bash
# Get last 50 messages from a chat
chat telegram get "My Group" --limit 50 --format text
```

### Example 2: Export chat history
```bash
# Export last 30 days to JSON
chat whatsapp export "Friend" --since "30 days ago" --format json --output backup.json
```

### Example 3: Real-time monitoring
```bash
# Watch for new messages
chat signal watch --all
```

### Example 4: Search across time range
```bash
# Search messages from last week
chat telegram search "Work Chat" "meeting" --since "7 days ago"
```

### Example 5: Complex filtering
```bash
# Get text messages from specific sender in last 3 days
chat whatsapp get "Group" \
  --sender "John" \
  --type text \
  --since "3 days ago" \
  --limit 100 \
  --format json
```

## Next Steps

1. **Review proposals:** Review each proposal in `openspec/changes/`
2. **Approve proposals:** Approve before implementation begins
3. **Choose platform:** Start with one platform (recommend Telegram for stability)
4. **Implement incrementally:** Follow tasks.md in each proposal
5. **Test thoroughly:** Test with real accounts (use test accounts for safety)

## File Locations

- **Telegram:** `openspec/changes/add-telegram-cli/`
- **WhatsApp:** `openspec/changes/add-whatsapp-cli/`
- **Signal:** `openspec/changes/add-signal-cli/`

Each contains:
- `proposal.md` - Why, what, impact
- `tasks.md` - Implementation checklist
- `specs/<platform>-cli/spec.md` - Detailed requirements and scenarios

## Validation Status

✅ All proposals validated with `openspec validate --strict`

```bash
openspec list
```

Output:
```
Changes:
  add-signal-cli             0/67 tasks
  add-telegram-cli           0/52 tasks
  add-whatsapp-cli           0/61 tasks
```
