# Proposal: Unify Chat API

**Change ID:** `unify-chat-api`
**Status:** Draft
**Created:** 2025-11-18

## Problem Statement

The current chat library has inconsistent naming and lacks a standardized interface for accessing messages across different platforms (Telegram, Signal, WhatsApp). Additionally, there is no MCP (Model Context Protocol) server interface to expose chat functionality to LLM agents.

Current issues:
1. Platform-specific CLI commands (e.g., `chat telegram list`, `chat signal list`) lack consistency
2. No unified API for "sources" abstraction (Telegram, Signal, WhatsApp as different sources)
3. Missing filter specifications for message queries
4. No MCP server to expose chat operations to agents
5. Commands use inconsistent terminology (platforms vs sources, chats vs conversations)

## Proposed Solution

Create a unified chat API with three layers:

### 1. Library Interface Redesign
- Introduce `ChatSource` concept (replaces `ChatPlatform`)
- Standardize operations: `sources`, `chats`, `groups`, `messages`
- Define `MessageFilter` specification for querying
- Maintain backward compatibility with existing `ChatClient` trait

### 2. CLI Restructuring
- New command structure: `chat <operation> <source>:<filter>`
- Examples:
  - `chat sources` - List all configured sources
  - `chat chats telegram` - List chats for Telegram source
  - `chat groups telegram` - List only group chats for Telegram
  - `chat messages telegram:Antti --since=1d` - Get messages with filter
  - `chat messages telegram:* --search="keyword"` - Search across all chats

### 3. MCP Server Integration
- Implement MCP server exposing chat operations as tools/resources
- Enable LLM agents to query messages, list chats, search conversations
- Provide streaming updates for new messages
- Follow MCP protocol specification for tool definitions

## Benefits

1. **Consistency**: Unified API across all chat platforms
2. **Extensibility**: Easy to add new sources (Discord, Slack, etc.)
3. **Agent Integration**: MCP server enables autonomous agent access
4. **Better UX**: Intuitive command structure with filter syntax
5. **Maintainability**: Single interface reduces code duplication

## Scope

**In Scope:**
- Unified chat library API with ChatSource abstraction
- CLI command restructuring with new syntax
- MCP server implementation for chat operations
- Message filter specification syntax
- Migration guide from old to new CLI commands

**Out of Scope:**
- Sending messages (read-only for V1)
- Real-time bidirectional messaging
- Media file handling beyond URLs
- Cross-platform message sync

## Dependencies

- Depends on existing `add-telegram-cli`, `add-signal-cli`, `add-whatsapp-cli` implementations
- Requires MCP SDK integration (new dependency)
- CLI restructuring may require coordination with existing command structure

## Open Questions

1. Should we deprecate old CLI commands or maintain both syntaxes?
2. How should message filters handle platform-specific features (Telegram channels vs Signal groups)?
3. Should MCP server run as separate process or embedded in chat library?
4. What's the pagination strategy for large message queries?

## Validation Criteria

- [ ] All three sources (Telegram, Signal, WhatsApp) work with unified API
- [ ] MCP server successfully exposes chat operations
- [ ] CLI commands follow new syntax and work across platforms
- [ ] Filter syntax supports common query patterns (time, sender, keywords)
- [ ] Documentation includes migration guide and examples
- [ ] Backward compatibility maintained for library consumers
