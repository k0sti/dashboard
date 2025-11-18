# Telegram CLI Specification

## ADDED Requirements

### Requirement: Initialize Telegram Connection

The CLI MUST support initializing and authenticating a Telegram connection with API credentials.

#### Scenario: First-time authentication

- **WHEN** user runs `chat telegram init` with API credentials
- **THEN** the CLI prompts for phone number and authentication code
- **AND** saves session data for future use

#### Scenario: Resume existing session

- **WHEN** user runs `chat telegram init` and session file exists
- **THEN** the CLI uses existing session without re-authentication

### Requirement: Check Connection Status

The CLI MUST allow users to check the current connection status.

#### Scenario: Connected status

- **WHEN** user runs `chat telegram status`
- **THEN** the CLI displays connection state, username, and phone number

#### Scenario: Disconnected status

- **WHEN** user runs `chat telegram status` while disconnected
- **THEN** the CLI indicates not connected and suggests running `init`

### Requirement: List All Chats

The CLI MUST list all available chats, groups, and channels.

#### Scenario: List all chats

- **WHEN** user runs `chat telegram list`
- **THEN** the CLI displays all chats with ID, title, and type (DM/Group/Channel)

#### Scenario: List with JSON output

- **WHEN** user runs `chat telegram list --format json`
- **THEN** the CLI outputs chat list in JSON format

#### Scenario: Filter by chat type

- **WHEN** user runs `chat telegram list --type group`
- **THEN** the CLI displays only group chats

### Requirement: Retrieve Messages from Chat

The CLI MUST retrieve messages from a specified chat by name or ID.

#### Scenario: Get recent messages by chat name

- **WHEN** user runs `chat telegram get "Chat Name"`
- **THEN** the CLI displays recent messages from that chat

#### Scenario: Get messages by chat ID

- **WHEN** user runs `chat telegram get --id 123456789`
- **THEN** the CLI displays messages from the chat with that ID

#### Scenario: Chat not found

- **WHEN** user runs `chat telegram get "Nonexistent Chat"`
- **THEN** the CLI displays an error indicating chat not found

### Requirement: Limit Messages by Count

The CLI MUST support limiting the number of messages retrieved.

#### Scenario: Limit message count

- **WHEN** user runs `chat telegram get "Chat" --limit 50`
- **THEN** the CLI retrieves at most 50 messages, starting from the most recent

#### Scenario: Default limit

- **WHEN** user runs `chat telegram get "Chat"` without `--limit`
- **THEN** the CLI retrieves at most 100 messages by default

### Requirement: Filter Messages by Time

The CLI MUST support filtering messages by timestamp or relative time.

#### Scenario: Messages since absolute timestamp

- **WHEN** user runs `chat telegram get "Chat" --since "2025-01-01T00:00:00Z"`
- **THEN** the CLI retrieves messages sent after that timestamp

#### Scenario: Messages since relative time

- **WHEN** user runs `chat telegram get "Chat" --since "2 days ago"`
- **THEN** the CLI retrieves messages sent in the last 2 days

#### Scenario: Messages before timestamp

- **WHEN** user runs `chat telegram get "Chat" --before "2025-01-15T00:00:00Z"`
- **THEN** the CLI retrieves messages sent before that timestamp

#### Scenario: Time range filter

- **WHEN** user runs `chat telegram get "Chat" --since "7 days ago" --before "2 days ago"`
- **THEN** the CLI retrieves messages sent between 7 and 2 days ago

### Requirement: Watch Messages in Real-Time

The CLI MUST support streaming new messages as they arrive.

#### Scenario: Watch chat for new messages

- **WHEN** user runs `chat telegram watch "Chat"`
- **THEN** the CLI displays new messages as they arrive in real-time

#### Scenario: Stop watching

- **WHEN** user presses Ctrl+C while watching
- **THEN** the CLI stops streaming and exits gracefully

### Requirement: Export Messages to File

The CLI MUST support exporting messages to various file formats.

#### Scenario: Export to JSON

- **WHEN** user runs `chat telegram export "Chat" --format json --output messages.json`
- **THEN** the CLI writes all messages to a JSON file

#### Scenario: Export to CSV

- **WHEN** user runs `chat telegram export "Chat" --format csv --output messages.csv`
- **THEN** the CLI writes messages to a CSV file with columns: timestamp, sender, content

#### Scenario: Export with filters

- **WHEN** user runs `chat telegram export "Chat" --since "30 days ago" --format json`
- **THEN** the CLI exports only messages from the last 30 days

### Requirement: Multiple Output Formats

The CLI MUST support multiple output formats for displaying messages.

#### Scenario: Text output format

- **WHEN** user runs `chat telegram get "Chat" --format text`
- **THEN** messages are displayed in human-readable text format

#### Scenario: JSON output format

- **WHEN** user runs `chat telegram get "Chat" --format json`
- **THEN** messages are displayed as JSON array

#### Scenario: Compact output format

- **WHEN** user runs `chat telegram get "Chat" --format compact`
- **THEN** messages are displayed in compact single-line format

### Requirement: Filter Messages by Sender

The CLI MUST support filtering messages by sender username or ID.

#### Scenario: Filter by sender username

- **WHEN** user runs `chat telegram get "Chat" --sender "@username"`
- **THEN** the CLI displays only messages from that sender

#### Scenario: Filter by sender ID

- **WHEN** user runs `chat telegram get "Chat" --sender-id 123456`
- **THEN** the CLI displays only messages from that sender

### Requirement: Filter Messages by Type

The CLI MUST support filtering messages by content type.

#### Scenario: Text messages only

- **WHEN** user runs `chat telegram get "Chat" --type text`
- **THEN** the CLI displays only text messages

#### Scenario: Media messages only

- **WHEN** user runs `chat telegram get "Chat" --type media`
- **THEN** the CLI displays only messages with images, videos, or files

#### Scenario: Multiple types

- **WHEN** user runs `chat telegram get "Chat" --type text,image`
- **THEN** the CLI displays text and image messages only

### Requirement: Configuration Management

The CLI MUST support managing configuration settings.

#### Scenario: Set configuration value

- **WHEN** user runs `chat telegram config set api_id 12345`
- **THEN** the CLI saves the API ID to configuration file

#### Scenario: Get configuration value

- **WHEN** user runs `chat telegram config get api_id`
- **THEN** the CLI displays the stored API ID

#### Scenario: List all configuration

- **WHEN** user runs `chat telegram config list`
- **THEN** the CLI displays all configuration settings (masking sensitive values)

### Requirement: Search Messages

The CLI MUST support searching messages by text content.

#### Scenario: Search in specific chat

- **WHEN** user runs `chat telegram search "Chat" "search term"`
- **THEN** the CLI displays messages containing the search term

#### Scenario: Search all chats

- **WHEN** user runs `chat telegram search --all "search term"`
- **THEN** the CLI searches across all chats and displays matching messages

#### Scenario: Case-insensitive search

- **WHEN** user runs `chat telegram search "Chat" "term" --ignore-case`
- **THEN** the CLI performs case-insensitive search

### Requirement: Get Chat Information

The CLI MUST display detailed information about a specific chat.

#### Scenario: Get chat details

- **WHEN** user runs `chat telegram info "Chat Name"`
- **THEN** the CLI displays chat ID, type, member count, and description

#### Scenario: Get chat by ID

- **WHEN** user runs `chat telegram info --id 123456789`
- **THEN** the CLI displays information for that chat ID
