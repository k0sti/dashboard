# Design Document: Autonomous Agent Dashboard

## Context

This is a greenfield project implementing a desktop GUI for managing and interacting with autonomous AI agents. The system needs to support multiple concurrent agent connections, unified chat interface, and extensible agent types. Initial target is local development use with Ollama LLMs, with potential for future expansion to cloud-based agents and additional toolcalls.

**Constraints:**
- Desktop application (cross-platform desirable)
- Low latency UI responsiveness
- Support for multiple concurrent agents (target: 5-10 agents)
- Safe shell command execution
- Persistent state across sessions

**Stakeholders:**
- Developers who want to interact with AI agents for coding assistance
- Users who need multi-agent orchestration capabilities
- Future: Teams collaborating with shared agent configurations

## Goals / Non-Goals

**Goals:**
- Create intuitive GUI for multi-agent interaction
- Support Ollama LLM agent type as first implementation
- Enable safe shell command execution from agents
- Provide unified chat experience across all agents
- Support agent-specific and broadcast messaging
- Implement basic planning/task tracking interface
- Ensure persistence of configurations, chat history, and plans

**Non-Goals:**
- Cloud synchronization (future consideration)
- Mobile/web interface (desktop-only for v1)
- Advanced security sandboxing (basic safety checks only)
- Distributed agent execution (local-only for v1)
- Custom LLM training or fine-tuning

## Decisions

### Technology Stack

**Decision: Rust + egui for implementation**

Rationale:
- Rust provides memory safety and performance for concurrent agent handling
- egui is immediate-mode GUI framework with good cross-platform support
- egui has minimal dependencies and produces native-feeling UIs
- Rust's async ecosystem (tokio) suits agent communication patterns
- Strong type system helps prevent runtime errors in complex state management

Alternatives considered:
- Electron/Tauri + React: More developer familiarity, but heavier runtime and harder to optimize
- Python + Qt: Rapid development, but performance concerns with multiple agents and GIL limitations
- Go + Fyne: Simpler language, but less mature GUI ecosystem

### Architecture Pattern

**Decision: Actor-based architecture for agents**

Each agent runs as an independent async task with message-passing interface. Main UI thread communicates with agents via channels (mpsc/broadcast).

Structure:
```
Main UI Thread (egui)
  ↓ (channels)
AgentManager
  ↓ (spawn)
Agent Tasks (async)
  ↓ (API calls)
Ollama / External Services
```

**Decision: Event-driven UI updates**

UI polls for agent messages on each frame (egui immediate mode). State changes trigger UI redraws automatically.

### Data Models

**Agent Trait:**
```rust
trait Agent {
    async fn send_message(&self, msg: String) -> Result<()>;
    fn get_status(&self) -> AgentStatus;
    fn get_id(&self) -> AgentId;
    fn get_config(&self) -> &AgentConfig;
}
```

**Message Structure:**
```rust
struct ChatMessage {
    id: MessageId,
    agent_id: Option<AgentId>, // None for user messages
    content: String,
    timestamp: DateTime,
    direction: MessageDirection, // ToAgent, FromAgent, Broadcast
    metadata: MessageMetadata, // toolcalls, errors, etc.
}
```

**Configuration Storage:**
- JSON/TOML files in user config directory (`~/.config/agent-dashboard/`)
- Separate files: `agents.json`, `chat_history.db` (SQLite), `plans.json`

### Shell Execution Safety

**Decision: Allowlist + timeout-based approach**

Initial safety mechanisms:
1. Timeout: 30 seconds default (configurable)
2. Working directory restriction: agents start in safe directory
3. Optional command allowlist/denylist patterns
4. Resource limits via OS-level controls (future: cgroups/job objects)

No sandboxing in v1, but log all commands for audit trail.

**Future enhancement:** Integrate with proper sandboxing (firejail, containers)

### Ollama Integration

**Decision: Use `ollama-rs` crate for API client**

If not available, implement custom HTTP client against Ollama REST API using `reqwest`.

**Streaming:** Use Server-Sent Events (SSE) for streaming responses, update UI incrementally.

**Toolcalls:** Implement function calling via Ollama's tools API (if supported) or custom prompt engineering.

## Risks / Trade-offs

### Risk: UI blocking on agent operations
- **Mitigation:** All agent I/O happens async, UI only reads from channels
- **Trade-off:** More complex async code, but maintains responsiveness

### Risk: Shell command security
- **Mitigation:** Timeout limits, command logging, future allowlist
- **Trade-off:** Not as secure as full sandbox, but simpler to implement
- **Acceptance:** V1 is developer tool, user is responsible for agent behavior

### Risk: Ollama API changes
- **Mitigation:** Abstract agent interface, version-specific adapters
- **Trade-off:** More abstraction layers, but easier to update

### Risk: Chat history database growth
- **Mitigation:** Implement retention policies (e.g., keep last 1000 messages per agent)
- **Trade-off:** Lose old history, but keep performance

### Risk: egui learning curve
- **Mitigation:** Start with simple layouts, reference examples
- **Trade-off:** Immediate mode is different paradigm, but leads to simpler state management

## Migration Plan

N/A - This is initial implementation.

**Future migration considerations:**
- If moving to web: abstract UI layer behind trait, implement web backend
- If adding cloud sync: implement sync protocol with conflict resolution
- If changing agent types: maintain backward compat in saved configs via versioning

## Open Questions

1. **Multi-user support:** Should configurations be system-wide or per-user?
   - **Resolution:** Per-user for v1, system-wide is future enhancement

2. **Plan interface complexity:** Should plans be first-class or embedded in chat?
   - **Resolution:** Separate panel for plans, but reference in chat. Keeps chat clean while plans are visible.

3. **Agent plugin system:** Should agent types be compiled-in or plugin-based?
   - **Resolution:** Compiled-in for v1 (Ollama only). Plugin system is v2 feature.

4. **Error recovery:** How should agents handle persistent failures?
   - **Resolution:** Exponential backoff for reconnects, user notification after 3 failures, manual reconnect option.

5. **Configuration UI location:** Separate window or integrated panel?
   - **Resolution:** Integrated panel (can toggle visibility). Keeps single window for simplicity.

## Implementation Phases

**Phase 1: Core UI & Framework** (Tasks 1-2)
- Basic egui window, layout, message display

**Phase 2: Configuration & Persistence** (Task 3, 4.1, 4.5)
- Agent config CRUD, save/load

**Phase 3: Ollama Integration** (Tasks 4.2-4.4, 6)
- Agent connection, Ollama client, chat

**Phase 4: Chat Interface** (Task 5)
- Full chat features, routing, history

**Phase 5: Shell Toolcalls** (Task 7)
- Command execution, safety checks

**Phase 6: Planning** (Task 8)
- Plan UI, agent integration

**Phase 7: Testing & Polish** (Task 9)
- Tests, error handling, docs
