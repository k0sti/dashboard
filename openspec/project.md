# Project Context

## Purpose

Autonomous Agent Dashboard is a desktop GUI application for managing and interacting with multiple AI agents simultaneously. The application provides a unified interface for configuring agents, conducting multi-agent conversations, executing tasks via toolcalls, and tracking agent plans. The goal is to create an intuitive workspace for orchestrating autonomous agents with access to operating system and LLM capabilities.

## Tech Stack

- **Language:** Rust (for performance, safety, and concurrency)
- **GUI Framework:** egui (immediate-mode, cross-platform)
- **Async Runtime:** Tokio (for agent task management)
- **LLM Integration:** Ollama (initial agent type)
- **Persistence:** JSON/TOML for configs, SQLite for chat history
- **Build Tool:** Cargo

## Project Conventions

### Code Style

- Follow Rust standard style (`rustfmt`)
- Use descriptive names for types and functions
- Prefer explicit error handling with `Result<T, E>`
- Document public APIs with doc comments (`///`)
- Keep modules focused and single-purpose

### Architecture Patterns

- **Actor model** for agent management (async tasks with message passing)
- **Trait-based abstractions** for agent types to support extensibility
- **Immediate-mode UI** with egui (state drives rendering each frame)
- **Event-driven communication** between UI and agents via channels (mpsc/broadcast)
- **Separation of concerns:** UI layer, agent layer, storage layer

### Testing Strategy

- Unit tests for core logic (agent traits, message routing, toolcall execution)
- Integration tests for agent workflows (send message, receive response)
- Manual testing for UI interactions and multi-agent scenarios
- Test with multiple concurrent agents to verify performance
- Test error cases (connection failures, command timeouts)

### Git Workflow

- Feature branches for new capabilities
- Commit messages should reference OpenSpec changes when applicable
- Use conventional commits format: `feat:`, `fix:`, `docs:`, `refactor:`
- Squash commits when merging to keep history clean

## Domain Context

### Agent Types
- Agents are autonomous entities that communicate via chat interface
- Each agent type implements the `Agent` trait
- Initial focus: Ollama-based LLM agents

### Toolcalls
- Agents can invoke toolcalls to interact with the system
- Toolcalls are registered capabilities (e.g., shell command execution)
- Toolcall results are returned to agents and displayed in chat

### Plans
- Plans are structured task lists created by agents or users
- Plans track step-by-step progress toward goals
- Associated with specific agents for execution tracking

### Chat Interface
- Unified log showing all agent messages chronologically
- Support for directed messages (to specific agent) and broadcasts (to all agents)
- Messages tagged with agent identifiers for clarity

## Important Constraints

- **Security:** Shell command execution requires safety checks (timeouts, validation)
- **Performance:** UI must remain responsive with 5-10 concurrent agents
- **Persistence:** Configuration and chat history must survive application restarts
- **Cross-platform:** Target Linux, macOS, and Windows (via egui/Rust)
- **Local-only:** V1 focuses on local execution, no cloud sync or distributed agents

## External Dependencies

- **Ollama API:** REST API for LLM interactions, streaming responses
  - Assumed to be running locally (default: `http://localhost:11434`)
- **Operating System:** Shell command execution uses OS-specific shells
  - Linux/macOS: `/bin/sh` or `/bin/bash`
  - Windows: `cmd.exe` or PowerShell (future)
