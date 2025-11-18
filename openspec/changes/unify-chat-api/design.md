# Design: Unify Chat API

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     MCP Server                          │
│  (Exposes chat operations as MCP tools/resources)       │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Unified Chat API                           │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Sources   │  │    Chats     │  │   Messages   │  │
│  │   Manager   │  │    Query     │  │    Filter    │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
      ┌──────────────┼──────────────┐
      │              │               │
┌─────▼─────┐  ┌────▼────┐  ┌──────▼──────┐
│ Telegram  │  │ Signal  │  │  WhatsApp   │
│  Source   │  │ Source  │  │   Source    │
└───────────┘  └─────────┘  └─────────────┘
```

## Core Components

### 1. ChatSource Trait

Replaces the current `ChatClient` trait with a more flexible source-based abstraction:

```rust
#[async_trait]
pub trait ChatSource: Send + Sync {
    /// Get source identifier (telegram, signal, whatsapp)
    fn source_id(&self) -> &str;

    /// Get display name for the source
    fn source_name(&self) -> &str;

    /// Check if source is connected
    fn is_connected(&self) -> bool;

    /// List all chats (conversations) from this source
    async fn list_chats(&self, filter: Option<ChatFilter>) -> Result<Vec<Chat>>;

    /// Get messages matching filter
    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>>;

    /// Subscribe to new messages (optional)
    async fn subscribe(&self) -> Result<Option<Receiver<Message>>>;
}
```

### 2. Filter Specifications

#### MessageFilter
```rust
pub struct MessageFilter {
    /// Source-specific chat ID or name pattern
    pub chat: ChatPattern,

    /// Time range
    pub since: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,

    /// Sender filter (name or ID pattern)
    pub sender: Option<String>,

    /// Text search
    pub search: Option<String>,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Message types
    pub content_type: Option<Vec<ContentType>>,
}

pub enum ChatPattern {
    /// Specific chat by ID
    Id(ChatId),

    /// Chat by name (partial match)
    Name(String),

    /// All chats
    All,

    /// Multiple specific chats
    Multiple(Vec<ChatId>),
}
```

#### ChatFilter
```rust
pub struct ChatFilter {
    /// Filter by chat type
    pub chat_type: Option<ChatType>,

    /// Name pattern matching
    pub name_pattern: Option<String>,

    /// Only include chats with recent activity
    pub active_since: Option<DateTime<Utc>>,
}
```

### 3. SourcesManager

Central registry for managing multiple chat sources:

```rust
pub struct SourcesManager {
    sources: HashMap<String, Box<dyn ChatSource>>,
}

impl SourcesManager {
    /// Register a new source
    pub fn register(&mut self, source: Box<dyn ChatSource>) -> Result<()>;

    /// Get source by ID
    pub fn get_source(&self, id: &str) -> Option<&dyn ChatSource>;

    /// List all registered sources
    pub fn list_sources(&self) -> Vec<SourceInfo>;

    /// Query messages across sources
    pub async fn query_messages(&self,
        source_id: Option<&str>,
        filter: MessageFilter
    ) -> Result<Vec<Message>>;
}
```

### 4. CLI Command Syntax

New command structure with filter syntax:

```
chat <operation> [source]:[filter] [options]
```

**Operations:**
- `sources` - List configured sources
- `chats` - List chats/conversations
- `groups` - List only group chats
- `messages` - Query messages

**Filter Syntax:**
- `telegram:Antti` - Specific chat by name
- `telegram:123456` - Specific chat by ID
- `telegram:*` - All chats in source
- `*:keyword` - Search across all sources

**Examples:**
```bash
# List all sources
chat sources

# List chats in Telegram
chat chats telegram

# List groups only
chat groups telegram

# Get messages from specific chat
chat messages telegram:Antti --limit 10 --since 7d

# Search across all Telegram chats
chat messages telegram:* --search "meeting"

# Get messages from all sources
chat messages "*:*" --since 1d

# Complex filter
chat messages telegram:* \
  --sender Alice \
  --search "project" \
  --since "2025-01-01" \
  --limit 100
```

### 5. MCP Server Design

#### Tools

**list_sources**
```json
{
  "name": "list_sources",
  "description": "List all configured chat sources",
  "inputSchema": {
    "type": "object",
    "properties": {}
  }
}
```

**list_chats**
```json
{
  "name": "list_chats",
  "description": "List chats from a source",
  "inputSchema": {
    "type": "object",
    "properties": {
      "source": {"type": "string"},
      "filter": {
        "type": "object",
        "properties": {
          "chat_type": {"type": "string", "enum": ["direct", "group", "channel"]},
          "name_pattern": {"type": "string"}
        }
      }
    },
    "required": ["source"]
  }
}
```

**get_messages**
```json
{
  "name": "get_messages",
  "description": "Query messages with filters",
  "inputSchema": {
    "type": "object",
    "properties": {
      "source": {"type": "string"},
      "chat": {"type": "string"},
      "since": {"type": "string", "format": "date-time"},
      "before": {"type": "string", "format": "date-time"},
      "sender": {"type": "string"},
      "search": {"type": "string"},
      "limit": {"type": "integer"}
    },
    "required": ["source"]
  }
}
```

#### Resources

**messages://{source}/{chat}**
- Provides message history as a resource
- Supports pagination via URI parameters
- Example: `messages://telegram/Antti?since=7d&limit=100`

#### Prompts

**analyze_conversation**
```json
{
  "name": "analyze_conversation",
  "description": "Analyze a chat conversation",
  "arguments": [
    {
      "name": "source",
      "description": "Chat source (telegram, signal, whatsapp)",
      "required": true
    },
    {
      "name": "chat",
      "description": "Chat name or ID",
      "required": true
    }
  ]
}
```

## Implementation Phases

### Phase 1: Core Library (Specs: unified-api, message-filters)
1. Define ChatSource trait
2. Implement MessageFilter and ChatFilter
3. Create SourcesManager
4. Migrate existing ChatClient implementations

### Phase 2: CLI Restructuring (Spec: cli-commands)
1. Implement new command parser with filter syntax
2. Update command handlers to use SourcesManager
3. Add backward compatibility layer
4. Update documentation and help text

### Phase 3: MCP Server (Spec: mcp-server)
1. Add MCP SDK dependency
2. Implement MCP tools (list_sources, list_chats, get_messages)
3. Implement MCP resources (messages://)
4. Add server startup and lifecycle management

### Phase 4: Integration & Testing
1. Integration tests for all sources
2. MCP server conformance tests
3. CLI command tests
4. Performance benchmarks

## Trade-offs & Decisions

### ChatSource vs ChatClient
**Decision:** Introduce ChatSource alongside ChatClient (not replacing)
**Rationale:**
- Maintain backward compatibility
- Allow gradual migration
- ChatClient remains for simple use cases
- ChatSource provides richer filtering and multi-source support

### Embedded vs Standalone MCP Server
**Decision:** Embedded server with optional standalone mode
**Rationale:**
- Easier deployment (no separate process management)
- Better integration with existing code
- Optional standalone for remote access
- Can run as systemd service if needed

### Filter Syntax: Structured vs String
**Decision:** Structured filters with optional string syntax for CLI
**Rationale:**
- Structured filters easier to validate and compose
- String syntax provides better CLI UX
- Parser converts string → structured filter
- MCP uses structured filters directly

### Backward Compatibility
**Decision:** Keep old commands for 2 releases, then deprecate
**Rationale:**
- Smooth migration path
- Users have time to update scripts
- Clear deprecation warnings
- Can detect old command usage

## Security Considerations

1. **Message Access Control**: MCP server should respect source-level permissions
2. **Filter Validation**: Prevent injection attacks in filter parsing
3. **Rate Limiting**: Prevent abuse of message queries
4. **Credential Storage**: Sources maintain their own credential management
5. **MCP Authentication**: Support token-based auth for MCP clients

## Performance Considerations

1. **Message Caching**: Cache recent messages to reduce API calls
2. **Lazy Loading**: Don't load all sources on startup
3. **Streaming**: Support streaming large result sets
4. **Pagination**: Implement cursor-based pagination for messages
5. **Concurrent Queries**: Allow parallel queries to different sources

## Migration Path

1. Add new API alongside existing code
2. Update internal code to use new API
3. Mark old CLI commands as deprecated
4. Provide migration guide
5. After 2 releases, remove old code
