## ADDED Requirements

### Requirement: Agent Connection Lifecycle
The system SHALL manage connections to configured agents, supporting connection establishment and termination.

#### Scenario: Connect to agent
- **WHEN** user selects a configured agent
- **THEN** system SHALL establish connection to the agent
- **AND** connection status SHALL be displayed
- **AND** agent SHALL appear in active connections list on failure

#### Scenario: Disconnect from agent
- **WHEN** user closes an active agent connection
- **THEN** system SHALL gracefully terminate the connection
- **AND** agent SHALL be removed from active connections list
- **AND** chat history with that agent SHALL be preserved

#### Scenario: Auto-reconnect on connection loss
- **WHEN** an active agent connection is lost unexpectedly
- **THEN** system SHALL attempt to reconnect automatically
- **AND** user SHALL be notified of reconnection attempts
- **AND** chat SHALL show connection status updates

### Requirement: Active Agent List
The system SHALL display a scrollable list of active agent connections in the left sidebar.

#### Scenario: Display active agents
- **WHEN** one or more agents are connected
- **THEN** each agent SHALL appear in the left sidebar
- **AND** list SHALL show agent name and connection status
- **AND** list SHALL support scrolling when many agents are active

#### Scenario: Select active agent
- **WHEN** user clicks an agent in the active list
- **THEN** that agent SHALL be highlighted as selected
- **AND** chat interface SHALL allow sending messages to that agent

### Requirement: Terminal Panel Interface
The system SHALL provide a terminal panel for each agent connection, initially supporting text-based chat.

#### Scenario: Display terminal panel
- **WHEN** an agent is connected
- **THEN** system SHALL display a terminal panel for that agent
- **AND** panel SHALL show chat interface
- **AND** panel SHALL support text input and output
