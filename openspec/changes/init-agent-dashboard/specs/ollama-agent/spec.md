## ADDED Requirements

### Requirement: Ollama Agent Type
The system SHALL provide an Ollama agent type that connects to Ollama API for LLM-powered conversations.

#### Scenario: Configure Ollama agent
- **WHEN** user creates a new Ollama agent
- **THEN** system SHALL prompt for Ollama server host/URL
- **AND** system SHALL prompt for model selection
- **AND** system SHALL validate connection to Ollama server
- **AND** configuration SHALL be saved on successful connection

#### Scenario: List available Ollama models
- **WHEN** configuring an Ollama agent
- **THEN** system SHALL query Ollama server for available models
- **AND** present model list to user for selection
- **AND** allow manual model name entry if list unavailable

### Requirement: Ollama Chat Interface
The system SHALL enable chat conversations with Ollama models through the unified chat interface.

#### Scenario: Send message to Ollama
- **WHEN** user sends a message to an Ollama agent
- **THEN** system SHALL transmit message to Ollama API
- **AND** maintain conversation context across messages
- **AND** display Ollama's response in the chat log

#### Scenario: Handle Ollama streaming responses
- **WHEN** Ollama API streams a response
- **THEN** system SHALL display response incrementally as it arrives
- **AND** update chat UI in real-time
- **AND** indicate when response is complete

#### Scenario: Handle Ollama errors
- **WHEN** Ollama API returns an error
- **THEN** system SHALL display error message in chat
- **AND** maintain agent connection for retry
- **AND** log error details for debugging

### Requirement: Ollama Toolcall Integration
The Ollama agent SHALL support toolcall capabilities exposed by the system.

#### Scenario: Register toolcalls with Ollama
- **WHEN** Ollama agent is initialized
- **THEN** system SHALL register available toolcalls (e.g., shell execution)
- **AND** provide toolcall schemas to Ollama API
- **AND** enable Ollama to invoke tools during conversation

#### Scenario: Execute Ollama toolcall request
- **WHEN** Ollama requests a toolcall execution
- **THEN** system SHALL validate the toolcall request
- **AND** execute the requested tool with provided parameters
- **AND** return execution results to Ollama
- **AND** display toolcall execution in chat log
