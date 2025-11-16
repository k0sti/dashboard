# Agent Dashboard

A desktop GUI application for managing and interacting with multiple autonomous AI agents. Built with Rust and egui.

## Features

- **Multi-Agent Management**: Configure and connect to multiple AI agents simultaneously
- **Unified Chat Interface**: Single chat view showing all agent conversations with message routing
- **Text-to-Speech**: Convert agent messages to speech with configurable voices and playback speed (V1: stub implementation)
- **Ollama Integration**: First-class support for Ollama LLM agents
- **Shell Toolcalls**: Agents can execute shell commands with safety timeouts
- **Planning Interface**: Track agent task plans and execution progress
- **Persistent Storage**: Configuration and chat history saved across sessions

## Prerequisites

- Rust 1.70+ (with Cargo)
- Ollama installed and running (default: http://localhost:11434)

## Installation

```bash
# Clone the repository
cd dashboard

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

### 1. Configure an Agent

1. Click the "Config" button in the top menu
2. Click "Add Agent" to create a new agent configuration
3. Default Ollama agent will be created with:
   - Host: http://localhost:11434
   - Model: llama2

### 2. Connect to an Agent

1. In the Config panel, click "Connect" next to an agent
2. The agent will appear in the left sidebar under "Active Agents"
3. Select the agent to send messages to it specifically

### 3. Chat with Agents

- **Send to specific agent**: Select an agent from the sidebar and type your message
- **Broadcast to all**: Click "📢 Broadcast" and messages will be sent to all active agents
- **Press Enter to send** (Shift+Enter for new line in message)

### 4. Use Text-to-Speech

1. Click the "TTS" button to open the TTS settings panel
2. Enable TTS with the checkbox
3. Configure:
   - Auto-speak: Automatically speak new agent messages
   - Playback speed: Adjust from 0.5x to 2.0x
   - Voice model: Select or enter voice model name
4. Click the 🔊 button on any message to speak it

**TTS Implementation**: The system generates audio using tone-based synthesis where each word is represented by a tone that varies in frequency and duration based on text characteristics. Audio is saved as WAV files in `~/.config/agent-dashboard/tts/audio/` for playback.

**Voice Models**: Different voice IDs affect the base pitch:
- IDs containing "low" → Lower pitch (~180Hz)
- IDs containing "high" → Higher pitch (~260Hz)
- Default → Medium pitch (~220Hz)

**Future Enhancement**: Optional Piper TTS integration will be available via the `tts` feature flag for neural TTS synthesis.

### 5. View Plans

1. Click the "Plans" button to show the plans panel
2. View agent task plans and their progress
3. Plans are associated with specific agents

## Project Structure

```
src/
├── agent/           # Agent trait and implementations
│   ├── types.rs     # Agent interfaces
│   └── ollama.rs    # Ollama agent implementation
├── config/          # Configuration management
├── plan/            # Planning data structures
├── storage/         # Persistent storage (SQLite)
├── tts/             # Text-to-Speech system
│   ├── config.rs    # TTS configuration
│   ├── model.rs     # Voice model management & synthesis
│   ├── synthesis.rs # Text preprocessing
│   ├── playback.rs  # WAV file generation & playback
│   ├── queue.rs     # TTS request queue
│   └── service.rs   # TTS service facade
├── toolcall/        # Toolcall system
│   ├── types.rs     # Toolcall interface
│   └── shell.rs     # Shell command execution
└── ui/              # User interface
    ├── app.rs       # Main application
    ├── chat.rs      # Chat message rendering
    └── config_panel.rs  # Configuration UI
```

## Configuration Files

Agent configurations are stored in:
- Linux: `~/.config/agent-dashboard/agents.json`
- macOS: `~/Library/Application Support/agent-dashboard/agents.json`
- Windows: `%APPDATA%\agent-dashboard\agents.json`

Chat history is stored in SQLite database at the same location.

### TTS Voice Models (Future)

When real Piper TTS is integrated, voice models will be stored in:
- Linux: `~/.config/agent-dashboard/tts/models/`
- macOS: `~/Library/Application Support/agent-dashboard/tts/models/`
- Windows: `%APPDATA%\agent-dashboard\tts\models\`

Place `.onnx` and `.json` model files in this directory.

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Check for errors
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

The application follows an actor-based architecture:

- **UI Thread**: egui immediate-mode GUI renders at 60fps
- **Agent Tasks**: Each agent runs as async task with message passing
- **Channels**: mpsc/broadcast channels for UI ↔ Agent communication
- **Storage**: SQLite for chat history, JSON for configuration

## Safety Features

Shell command execution includes:
- 30-second timeout (configurable)
- Command logging for audit trail
- Working directory restrictions
- Exit code tracking

## Extending

### Adding New Agent Types

1. Implement the `Agent` trait in `src/agent/`
2. Add agent type to `AgentType` enum
3. Register in configuration panel

### Adding New Toolcalls

1. Implement the `Toolcall` trait in `src/toolcall/`
2. Register in `ToolcallRegistry`
3. Expose schema to agents

## License

This project is part of the OpenSpec initiative.

## Contributing

This implementation follows the OpenSpec workflow. See `openspec/changes/init-agent-dashboard/` for the complete specification.
