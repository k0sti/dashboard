# Tasks: Unify Chat API

## Phase 1: Core Library (unified-api, message-filters)

### 1.1 Define Core Traits and Types
- [ ] Create `ChatSource` trait with methods: source_id, source_name, is_connected, list_chats, get_messages, subscribe
- [ ] Define `MessageFilter` struct with fields: chat, since, before, sender, search, limit, content_type
- [ ] Define `ChatFilter` struct with fields: chat_type, name_pattern, active_since
- [ ] Define `ChatPattern` enum with variants: Id, Name, All, Multiple
- [ ] Add validation methods to MessageFilter and ChatFilter
- [ ] Write unit tests for filter validation

### 1.2 Implement SourcesManager
- [ ] Create `SourcesManager` struct with HashMap of sources
- [ ] Implement register() method for adding sources
- [ ] Implement get_source() method for retrieving by ID
- [ ] Implement list_sources() method returning SourceInfo
- [ ] Implement query_messages() for cross-source queries
- [ ] Add interior mutability support (Arc<RwLock<>>)
- [ ] Write unit tests for SourcesManager

### 1.3 Create Telegram ChatSource Implementation
- [ ] Create TelegramSource struct wrapping existing client
- [ ] Implement ChatSource trait for TelegramSource
- [ ] Map Telegram-specific types to common types
- [ ] Implement list_chats with ChatFilter support
- [ ] Implement get_messages with MessageFilter support
- [ ] Implement subscribe using existing stream_updates
- [ ] Write integration tests with real Telegram API
- [ ] Update TELEGRAM_IMPLEMENTATION.md

### 1.4 Create Signal ChatSource Implementation
- [ ] Create SignalSource struct (stubbed initially)
- [ ] Implement ChatSource trait for SignalSource
- [ ] Add note about Signal implementation being future work
- [ ] Create placeholder tests

### 1.5 Create WhatsApp ChatSource Implementation
- [ ] Create WhatsAppSource struct (stubbed initially)
- [ ] Implement ChatSource trait for WhatsAppSource
- [ ] Add note about WhatsApp implementation being future work
- [ ] Create placeholder tests

### 1.6 Maintain Backward Compatibility
- [ ] Keep existing ChatClient trait unchanged
- [ ] Create adapter from ChatClient to ChatSource
- [ ] Ensure existing code continues to work
- [ ] Add deprecation notices to old code

## Phase 2: CLI Restructuring (cli-commands)

### 2.1 Implement Filter Parser
- [ ] Create filter_parser module
- [ ] Parse `source:pattern` syntax
- [ ] Parse time specifications (7d, 2h, ISO dates)
- [ ] Parse wildcard `*` for all sources/chats
- [ ] Handle quoted strings in search terms
- [ ] Write parser tests for edge cases

### 2.2 Implement Sources Command
- [ ] Add `sources` subcommand to CLI
- [ ] Display source ID, name, connection status
- [ ] Add colored output for connected/disconnected
- [ ] Support --format flag (text, json, csv)
- [ ] Write CLI tests

### 2.3 Implement Chats Command
- [ ] Add `chats <source>` subcommand
- [ ] Support --name filter for name pattern
- [ ] Support --type filter for chat type
- [ ] Support --active-since for recent activity
- [ ] Add table formatting for text output
- [ ] Support --format flag
- [ ] Write CLI tests

### 2.4 Implement Groups Command
- [ ] Add `groups <source>` subcommand
- [ ] Reuse chats command with chat_type=Group filter
- [ ] Support all formatting options
- [ ] Write CLI tests

### 2.5 Implement Messages Command
- [ ] Add `messages <source:pattern>` subcommand
- [ ] Parse all filter options (--since, --before, --sender, --search, --limit)
- [ ] Support wildcard patterns (*:* for all)
- [ ] Implement cross-source queries
- [ ] Add progress indicator for large queries
- [ ] Support all output formats
- [ ] Write CLI tests

### 2.6 Update Help and Documentation
- [ ] Add examples to --help output
- [ ] Create CLI usage guide
- [ ] Update README with new command syntax
- [ ] Add migration guide from old to new commands

### 2.7 Add Backward Compatibility Layer
- [ ] Keep old `telegram list` commands working
- [ ] Add deprecation warnings to old commands
- [ ] Log usage of deprecated commands for metrics
- [ ] Plan removal timeline (2 releases)

## Phase 3: MCP Server (mcp-server)

### 3.1 Add MCP Dependencies
- [ ] Add mcp-sdk crate to Cargo.toml
- [ ] Add required dependencies (serde_json, tokio)
- [ ] Configure features for stdio and TCP modes

### 3.2 Implement MCP Tool: list_sources
- [ ] Define tool schema (JSON Schema)
- [ ] Implement tool handler
- [ ] Return sources with connection status
- [ ] Add error handling
- [ ] Write tool tests

### 3.3 Implement MCP Tool: list_chats
- [ ] Define tool schema with filter parameters
- [ ] Implement tool handler
- [ ] Support chat_type and name_pattern filters
- [ ] Add error handling (source not found, not connected)
- [ ] Write tool tests

### 3.4 Implement MCP Tool: get_messages
- [ ] Define tool schema with all filter parameters
- [ ] Implement tool handler
- [ ] Support all MessageFilter options
- [ ] Add pagination support
- [ ] Add error handling
- [ ] Write tool tests

### 3.5 Implement MCP Resource: messages://
- [ ] Define resource URI template
- [ ] Implement resource handler
- [ ] Parse query parameters (since, limit, offset)
- [ ] Format messages as text
- [ ] Add cursor-based pagination
- [ ] Write resource tests

### 3.6 Implement MCP Prompt: analyze_conversation
- [ ] Define prompt schema with arguments
- [ ] Implement prompt handler
- [ ] Fetch recent messages (last 100)
- [ ] Generate analysis prompt with context
- [ ] Write prompt tests

### 3.7 Implement Server Lifecycle
- [ ] Add server initialization
- [ ] Implement stdio mode (embedded)
- [ ] Implement TCP mode (standalone)
- [ ] Add graceful shutdown handling (SIGTERM/SIGINT)
- [ ] Add connection timeout (30s)
- [ ] Add request timeout (60s)
- [ ] Write lifecycle tests

### 3.8 Implement Error Handling
- [ ] Define error codes (SOURCE_NOT_FOUND, CHAT_NOT_FOUND, etc.)
- [ ] Implement MCP error response format
- [ ] Add helpful error messages
- [ ] Add error logging
- [ ] Write error handling tests

### 3.9 Implement Authentication (Optional)
- [ ] Add token-based authentication
- [ ] Skip auth in stdio mode
- [ ] Support auth token in config
- [ ] Add auth middleware
- [ ] Write auth tests

### 3.10 Implement Streaming (Optional)
- [ ] Add subscription support for sources that support it
- [ ] Implement message streaming protocol
- [ ] Filter streamed messages
- [ ] Handle source disconnection
- [ ] Write streaming tests

### 3.11 Server Discovery
- [ ] Implement MCP initialize request
- [ ] Return capabilities list
- [ ] Return tools list with schemas
- [ ] Return resource templates
- [ ] Write discovery tests

## Phase 4: Integration & Testing

### 4.1 Integration Tests
- [ ] Test SourcesManager with all sources
- [ ] Test cross-source queries
- [ ] Test filter composition
- [ ] Test error propagation
- [ ] Test concurrent queries

### 4.2 MCP Server Integration Tests
- [ ] Test MCP server with real client
- [ ] Test all tools end-to-end
- [ ] Test resources with pagination
- [ ] Test prompts
- [ ] Test error scenarios

### 4.3 CLI Integration Tests
- [ ] Test all CLI commands end-to-end
- [ ] Test filter parsing edge cases
- [ ] Test output formats
- [ ] Test error messages

### 4.4 Performance Testing
- [ ] Benchmark message queries (1k, 10k, 100k messages)
- [ ] Benchmark cross-source queries
- [ ] Test concurrent MCP requests (10, 50, 100)
- [ ] Test memory usage with large result sets
- [ ] Optimize slow paths

### 4.5 Documentation
- [ ] Complete API documentation (rustdoc)
- [ ] Write CLI user guide
- [ ] Write MCP server setup guide
- [ ] Create architecture diagrams
- [ ] Add code examples
- [ ] Write migration guide

### 4.6 Final Validation
- [ ] Verify all acceptance criteria met
- [ ] Run full test suite
- [ ] Check backward compatibility
- [ ] Validate MCP conformance
- [ ] Code review

## Dependencies Between Tasks

- Phase 1 must complete before Phase 2 and 3
- Phase 2 and 3 can be done in parallel
- Phase 4 requires Phase 1, 2, and 3 to be complete
- Within each phase, tasks can be parallelized within reason
- Core traits (1.1) must be done before implementations (1.2-1.6)
- Filter parser (2.1) should be done before CLI commands (2.2-2.5)
- MCP dependencies (3.1) must be done before MCP implementations (3.2-3.11)

## Estimated Timeline

- Phase 1: 2-3 weeks (core library is foundation)
- Phase 2: 1-2 weeks (CLI restructuring)
- Phase 3: 2-3 weeks (MCP server is new territory)
- Phase 4: 1-2 weeks (testing and documentation)
- **Total: 6-10 weeks**

## Success Metrics

- [ ] All tests passing (unit, integration, end-to-end)
- [ ] 80%+ code coverage
- [ ] All OpenSpec scenarios validated
- [ ] MCP server conformance tests passing
- [ ] Performance benchmarks meet targets
- [ ] Documentation complete and reviewed
- [ ] Migration guide tested with real code
