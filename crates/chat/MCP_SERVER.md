# Chat MCP Server

Model Context Protocol (MCP) server for accessing chat messages from Telegram, Signal, and WhatsApp.

## Overview

The Chat MCP Server exposes chat operations to AI assistants through the Model Context Protocol. It provides three main tools for querying messages and managing sources.

## Installation

### Build the Server

```bash
cd crates/chat
cargo build --release --features mcp --bin chat-mcp-server
```

The binary will be located at: `target/release/chat-mcp-server`

## Tools

### 1. list_sources

List all configured chat sources with their connection status.

**Parameters:** None

**Returns:**
```json
{
  "sources": [
    {
      "id": "telegram",
      "name": "Telegram",
      "is_connected": true
    }
  ]
}
```

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_sources",
    "arguments": {}
  }
}
```

### 2. list_chats

List chats from a specific source with optional filtering.

**Parameters:**
- `source` (required): Source ID (telegram, signal, whatsapp)
- `name_pattern` (optional): Filter by name (case-insensitive substring)
- `chat_type` (optional): Filter by type (direct, group, channel)

**Returns:**
```json
{
  "chats": [
    {
      "id": "123456",
      "title": "Work Group",
      "chat_type": "group",
      "participant_count": 5
    }
  ]
}
```

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "list_chats",
    "arguments": {
      "source": "telegram",
      "name_pattern": "Work",
      "chat_type": "group"
    }
  }
}
```

### 3. get_messages

Get messages from a chat with advanced filtering.

**Parameters:**
- `chat` (required): Chat identifier (name, ID, or pattern like "Antti" or "*" for all)
- `source` (optional): Source ID - queries all sources if not specified
- `since` (optional): Messages after this time (e.g., "7d", "2h", "2025-01-15")
- `before` (optional): Messages before this time
- `sender` (optional): Filter by sender name or ID
- `search` (optional): Text search (case-insensitive substring)
- `limit` (optional): Limit number of results (default: 100)

**Returns:**
```json
{
  "messages": [
    {
      "id": "789",
      "chat_id": "123456",
      "sender": {
        "id": "456",
        "display_name": "Alice"
      },
      "content": "Hello world!",
      "timestamp": "2025-01-19T12:00:00Z",
      "edited": false
    }
  ],
  "total": 1
}
```

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_messages",
    "arguments": {
      "source": "telegram",
      "chat": "Antti",
      "since": "7d",
      "limit": 10
    }
  }
}
```

## Claude Desktop Configuration

To use this MCP server with Claude Desktop, add it to your `claude_desktop_config.json`:

### macOS

Location: `~/Library/Application Support/Claude/claude_desktop_config.json`

### Windows

Location: `%APPDATA%\Claude\claude_desktop_config.json`

### Configuration

```json
{
  "mcpServers": {
    "chat": {
      "command": "/path/to/chat-mcp-server",
      "args": []
    }
  }
}
```

Replace `/path/to/chat-mcp-server` with the actual path to your compiled binary.

Example on macOS:
```json
{
  "mcpServers": {
    "chat": {
      "command": "/Users/yourname/work/dashboard/crates/chat/target/release/chat-mcp-server",
      "args": []
    }
  }
}
```

## Usage in Claude

Once configured, Claude can use these tools automatically. Example prompts:

### List Available Sources

"List all my configured chat sources"

Claude will call `list_sources` and show you Telegram, Signal, or WhatsApp if configured.

### List Chats

"Show me all my Telegram group chats"

Claude will call `list_chats` with:
```json
{
  "source": "telegram",
  "chat_type": "group"
}
```

### Query Messages

"Get my last 10 messages from the chat with Antti from the last week"

Claude will call `get_messages` with:
```json
{
  "source": "telegram",
  "chat": "Antti",
  "since": "7d",
  "limit": 10
}
```

### Search Messages

"Search for messages containing 'meeting' in all my Telegram chats"

Claude will call `get_messages` with:
```json
{
  "source": "telegram",
  "chat": "*",
  "search": "meeting"
}
```

## Prerequisites

Before using the MCP server, you need to configure at least one chat source:

### Telegram Setup

```bash
# Initialize Telegram connection
chat telegram init

# This will prompt for:
# - API ID and API Hash (get from https://my.telegram.org)
# - Phone number
# - Verification code
# - 2FA password (if enabled)
```

Once configured, the session is stored in `~/.config/chat/telegram_session.sqlite` and the MCP server will automatically connect to it.

## Testing

You can manually test the MCP server using stdio:

```bash
# Start the server
./target/release/chat-mcp-server

# Send initialize request (in another terminal or via echo)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/chat-mcp-server

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./target/release/chat-mcp-server

# Call a tool
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_sources","arguments":{}}}' | ./target/release/chat-mcp-server
```

## Troubleshooting

### "Source not found" Error

Make sure you've initialized the chat source first:
```bash
chat telegram init
```

### "Not connected" Error

The source exists but is not connected. Try reconnecting:
```bash
chat telegram status
```

### MCP Server Not Appearing in Claude

1. Check that the path in `claude_desktop_config.json` is correct
2. Restart Claude Desktop
3. Check the binary has execute permissions: `chmod +x chat-mcp-server`
4. Check Claude Desktop logs for errors

### Debugging

The MCP server logs to stderr (not stdout, which is used for JSON-RPC). To see logs:

```bash
./target/release/chat-mcp-server 2>server.log
```

Then check `server.log` for debug information.

## Architecture

The MCP server:
1. Listens on stdin for JSON-RPC requests
2. Writes JSON-RPC responses to stdout
3. Uses the unified ChatSource API from Phase 1
4. Leverages the filter system from Phase 2
5. Maintains thread-safe access to sources via SourcesManager

## Limitations

- Currently supports stdio transport only (no HTTP/SSE)
- Sources must be pre-configured using the CLI
- No authentication (relies on filesystem permissions)
- Streaming updates not yet implemented
- Signal and WhatsApp sources are placeholders

## Future Enhancements

- MCP resources: `messages://{source}/{chat}`
- MCP prompts: `analyze_conversation`
- HTTP/SSE transport for remote access
- Real-time message streaming via subscriptions
- Source configuration management via MCP
- Signal and WhatsApp implementations

## License

Same as the parent project.
