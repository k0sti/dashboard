# Spec: MCP Server

**Capability:** `mcp-server`
**Status:** Draft

## Overview

Defines the Model Context Protocol (MCP) server interface for exposing chat operations to LLM agents as tools, resources, and prompts.

## ADDED Requirements

### Requirement: MCP Server Lifecycle

The system SHALL provide an MCP server that can be started, stopped, and managed.

#### Scenario: Start embedded server

**Given** chat library is initialized
**When** user starts MCP server in embedded mode
**Then** server listens on stdio
**And** accepts MCP protocol messages

#### Scenario: Start standalone server

**Given** chat library is initialized
**When** user starts MCP server with `--port=3000`
**Then** server listens on TCP port 3000
**And** accepts MCP protocol messages over HTTP

#### Scenario: Graceful shutdown

**Given** MCP server is running
**When** user sends SIGTERM signal
**Then** server completes in-flight requests
**And** closes all connections
**And** exits cleanly

### Requirement: Tool - list_sources

The system SHALL provide an MCP tool to list all configured chat sources.

#### Scenario: Call list_sources tool

**Given** Telegram and Signal are configured
**When** agent calls `list_sources` tool
**Then** server returns array of source objects
**And** each object contains: id, name, is_connected

#### Scenario: Include connection status

**Given** Telegram is connected but Signal is not
**When** agent calls `list_sources` tool
**Then** Telegram object has `is_connected: true`
**And** Signal object has `is_connected: false`

### Requirement: Tool - list_chats

The system SHALL provide an MCP tool to list chats from a source.

#### Scenario: List all chats from source

**Given** Telegram has multiple chats
**When** agent calls `list_chats` with `{"source": "telegram"}`
**Then** server returns array of chat objects
**And** each object contains: id, name, type, participant_count

#### Scenario: Filter chats by type

**Given** Telegram has Direct and Group chats
**When** agent calls `list_chats` with `{"source": "telegram", "filter": {"chat_type": "group"}}`
**Then** server returns only Group chats

#### Scenario: Filter chats by name pattern

**Given** Telegram has chats "Work", "Family", "Friends"
**When** agent calls `list_chats` with `{"source": "telegram", "filter": {"name_pattern": "Fam"}}`
**Then** server returns only "Family" chat

### Requirement: Tool - get_messages

The system SHALL provide an MCP tool to query messages with filters.

#### Scenario: Get messages from specific chat

**Given** Telegram chat "Antti" exists
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Antti"}`
**Then** server returns array of message objects
**And** each message contains: id, chat_id, sender, content, timestamp

#### Scenario: Filter messages by time range

**Given** chat "Antti" has messages from last 30 days
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Antti", "since": "2025-01-13T00:00:00Z"}`
**Then** server returns only messages from 2025-01-13 onwards

#### Scenario: Filter messages by sender

**Given** chat "Antti" has messages from multiple people
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Antti", "sender": "Alice"}`
**Then** server returns only messages from Alice

#### Scenario: Search messages by text

**Given** chat "Antti" has various messages
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Antti", "search": "meeting"}`
**Then** server returns only messages containing "meeting"

#### Scenario: Limit number of results

**Given** chat "Antti" has 1000 messages
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Antti", "limit": 10}`
**Then** server returns exactly 10 most recent messages

#### Scenario: Query all chats in source

**Given** Telegram has multiple chats
**When** agent calls `get_messages` with `{"source": "telegram", "search": "project"}`
**Then** server searches all chats in Telegram
**And** returns matching messages with chat identification

### Requirement: Resource - messages://

The system SHALL provide MCP resources for accessing message history.

#### Scenario: Read messages resource

**Given** Telegram chat "Antti" exists
**When** agent reads resource `messages://telegram/Antti`
**Then** server returns message history as text
**And** format is: `[timestamp] sender: content`

#### Scenario: Resource with query parameters

**Given** chat "Antti" has messages
**When** agent reads resource `messages://telegram/Antti?since=7d&limit=100`
**Then** server returns last 7 days of messages (max 100)
**And** applies filters from query parameters

#### Scenario: Resource pagination

**Given** chat "Antti" has 1000 messages
**When** agent reads resource `messages://telegram/Antti?limit=100&offset=100`
**Then** server returns messages 101-200
**And** supports cursor-based pagination

### Requirement: Prompt - analyze_conversation

The system SHALL provide an MCP prompt for conversation analysis.

#### Scenario: Get analysis prompt

**Given** chat "Antti" exists in Telegram
**When** agent calls prompt `analyze_conversation` with `{"source": "telegram", "chat": "Antti"}`
**Then** server returns prompt with recent messages
**And** prompt asks agent to analyze conversation patterns

#### Scenario: Prompt includes context

**Given** chat "Antti" has 1000 messages
**When** agent calls prompt `analyze_conversation`
**Then** server includes last 100 messages in prompt
**And** provides chat metadata (participants, type, name)

### Requirement: Error Handling

The MCP server SHALL return proper error responses.

#### Scenario: Invalid source

**Given** only Telegram is configured
**When** agent calls `list_chats` with `{"source": "signal"}`
**Then** server returns MCP error response
**And** error code is "SOURCE_NOT_FOUND"
**And** error message is "Source 'signal' not found"

#### Scenario: Chat not found

**Given** Telegram has no chat "Invalid"
**When** agent calls `get_messages` with `{"source": "telegram", "chat": "Invalid"}`
**Then** server returns MCP error response
**And** error code is "CHAT_NOT_FOUND"
**And** error message is "Chat 'Invalid' not found in source 'telegram'"

#### Scenario: Source not connected

**Given** Signal is configured but disconnected
**When** agent calls `list_chats` with `{"source": "signal"}`
**Then** server returns MCP error response
**And** error code is "SOURCE_NOT_CONNECTED"
**And** error message includes connection instructions

#### Scenario: Invalid filter parameters

**Given** agent calls get_messages
**When** agent provides `{"since": "invalid-date"}`
**Then** server returns MCP error response
**And** error code is "INVALID_PARAMETER"
**And** error message explains valid date formats

### Requirement: Server Discovery

The system SHALL implement MCP server discovery protocol.

#### Scenario: List capabilities

**Given** MCP server is running
**When** client sends `initialize` request
**Then** server responds with capabilities list
**And** includes: tools, resources, prompts

#### Scenario: List tools

**Given** MCP server is running
**When** client requests tool list
**Then** server returns: list_sources, list_chats, get_messages
**And** each tool includes schema and description

#### Scenario: List resources

**Given** MCP server is running
**When** client requests resource templates
**Then** server returns: messages://{source}/{chat}
**And** includes URI template and supported parameters

### Requirement: Authentication

The system SHALL support optional authentication for MCP clients.

#### Scenario: Token-based authentication

**Given** server is configured with auth enabled
**When** client connects without token
**Then** server rejects connection
**And** returns authentication required error

#### Scenario: Valid token

**Given** server is configured with token "secret123"
**When** client connects with token "secret123"
**Then** server accepts connection
**And** allows tool calls

#### Scenario: No authentication in embedded mode

**Given** server runs in embedded stdio mode
**When** client connects via stdio
**Then** server accepts connection without authentication

### Requirement: Streaming Updates

The system SHALL support streaming new messages to MCP clients.

#### Scenario: Subscribe to message stream

**Given** Telegram source supports streaming
**When** agent subscribes to `messages://telegram/*`
**Then** server sends new messages as they arrive
**And** each message includes source and chat identification

#### Scenario: Filter streaming messages

**Given** agent is subscribed to messages
**When** agent specifies filter `{"chat": "Antti"}`
**Then** server only streams messages from "Antti" chat

#### Scenario: Source doesn't support streaming

**Given** WhatsApp source doesn't support streaming
**When** agent tries to subscribe to `messages://whatsapp/*`
**Then** server returns error indicating streaming not supported

## Dependencies

- Depends on `unified-api` for ChatSource interface
- Depends on `message-filters` for filter parsing
- Requires MCP SDK (Rust implementation)
- Needs tokio for async runtime
- Requires serde_json for JSON serialization

## Implementation Notes

- Server should support both stdio (embedded) and TCP (standalone) modes
- Default mode: stdio for embedded usage
- Tool schemas should follow JSON Schema specification
- Resource URIs follow `messages://{source}/{chat}` pattern
- Query parameters: since, before, sender, search, limit, offset
- Error codes should be uppercase with underscores
- Streaming uses MCP subscription protocol
- Authentication is optional and disabled by default
- Server should handle graceful shutdown on SIGTERM/SIGINT
- Connection timeout: 30 seconds
- Request timeout: 60 seconds
- Maximum concurrent requests: 100
