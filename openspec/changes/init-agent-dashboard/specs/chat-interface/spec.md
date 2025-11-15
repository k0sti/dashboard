## ADDED Requirements

### Requirement: Unified Chat Log
The system SHALL maintain a unified chat log showing messages from all active agents in a single scrollable view.

#### Scenario: Display all agent messages
- **WHEN** multiple agents are active and sending messages
- **THEN** all messages SHALL appear in the central chat panel
- **AND** each message SHALL be tagged with the agent identifier
- **AND** messages SHALL be displayed in chronological order
- **AND** chat SHALL auto-scroll to newest messages

#### Scenario: Message visual distinction
- **WHEN** viewing the chat log
- **THEN** messages from different agents SHALL be visually distinguishable
- **AND** user messages SHALL be visually distinct from agent messages
- **AND** agent identifier SHALL be clearly visible for each message

### Requirement: User Input Interface
The system SHALL provide a user input bar at the bottom of the chat panel for composing messages.

#### Scenario: Send message to specific agent
- **WHEN** user has selected a specific agent from the active list
- **AND** user types a message and presses send
- **THEN** message SHALL be sent only to the selected agent
- **AND** message SHALL appear in the unified chat log with "to: [agent]" indicator

#### Scenario: Broadcast message to all agents
- **WHEN** user selects broadcast mode
- **AND** user types a message and presses send
- **THEN** message SHALL be sent to all active agents
- **AND** message SHALL appear in chat log with "to: all" indicator
- **AND** each agent SHALL receive and process the message independently

#### Scenario: Input bar state management
- **WHEN** user is typing a message
- **THEN** input bar SHALL show current recipient (specific agent or broadcast)
- **AND** user SHALL be able to change recipient before sending
- **AND** input bar SHALL support multi-line messages

### Requirement: Chat History Persistence
The system SHALL preserve chat history for each agent across sessions.

#### Scenario: Save chat history
- **WHEN** agents send or receive messages
- **THEN** messages SHALL be saved to persistent storage
- **AND** history SHALL be associated with specific agent configurations

#### Scenario: Load chat history
- **WHEN** reconnecting to a previously used agent
- **THEN** system SHALL load and display previous chat history
- **AND** new messages SHALL append to existing history
