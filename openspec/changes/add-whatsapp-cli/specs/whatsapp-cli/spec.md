# WhatsApp CLI Specification

## ADDED Requirements

### Requirement: Initialize WhatsApp Connection

The CLI MUST support initializing and authenticating a WhatsApp connection via QR code.

#### Scenario: First-time QR code authentication

- **WHEN** user runs `chat whatsapp init`
- **THEN** the CLI displays a QR code in the terminal
- **AND** waits for user to scan with WhatsApp mobile app
- **AND** saves session data for future use

#### Scenario: Resume existing session

- **WHEN** user runs `chat whatsapp init` and session file exists
- **THEN** the CLI uses existing session without re-authentication

#### Scenario: Session expired

- **WHEN** user runs any command with an expired session
- **THEN** the CLI prompts user to run `init` again

### Requirement: Check Connection Status

The CLI MUST allow users to check the current connection status.

#### Scenario: Connected status

- **WHEN** user runs `chat whatsapp status`
- **THEN** the CLI displays connection state and phone number

#### Scenario: Disconnected status

- **WHEN** user runs `chat whatsapp status` while disconnected
- **THEN** the CLI indicates not connected and suggests running `init`

### Requirement: List All Chats

The CLI MUST list all available chats and groups.

#### Scenario: List all chats

- **WHEN** user runs `chat whatsapp list`
- **THEN** the CLI displays all chats with ID, name, and type (individual/group)

#### Scenario: List with JSON output

- **WHEN** user runs `chat whatsapp list --format json`
- **THEN** the CLI outputs chat list in JSON format

#### Scenario: Filter by chat type

- **WHEN** user runs `chat whatsapp list --type group`
- **THEN** the CLI displays only group chats

### Requirement: Retrieve Messages from Chat

The CLI MUST retrieve messages from a specified chat by name or phone number.

#### Scenario: Get recent messages by contact name

- **WHEN** user runs `chat whatsapp get "Contact Name"`
- **THEN** the CLI displays recent messages from that chat

#### Scenario: Get messages by phone number

- **WHEN** user runs `chat whatsapp get "+1234567890"`
- **THEN** the CLI displays messages from that contact

#### Scenario: Get messages by chat ID

- **WHEN** user runs `chat whatsapp get --id "123456789@c.us"`
- **THEN** the CLI displays messages from the chat with that ID

#### Scenario: Chat not found

- **WHEN** user runs `chat whatsapp get "Nonexistent Chat"`
- **THEN** the CLI displays an error indicating chat not found

### Requirement: Limit Messages by Count

The CLI MUST support limiting the number of messages retrieved.

#### Scenario: Limit message count

- **WHEN** user runs `chat whatsapp get "Contact" --limit 50`
- **THEN** the CLI retrieves at most 50 messages, starting from the most recent

#### Scenario: Default limit

- **WHEN** user runs `chat whatsapp get "Contact"` without `--limit`
- **THEN** the CLI retrieves at most 100 messages by default

### Requirement: Filter Messages by Time

The CLI MUST support filtering messages by timestamp or relative time.

#### Scenario: Messages since absolute timestamp

- **WHEN** user runs `chat whatsapp get "Contact" --since "2025-01-01T00:00:00Z"`
- **THEN** the CLI retrieves messages sent after that timestamp

#### Scenario: Messages since relative time

- **WHEN** user runs `chat whatsapp get "Contact" --since "2 days ago"`
- **THEN** the CLI retrieves messages sent in the last 2 days

#### Scenario: Messages before timestamp

- **WHEN** user runs `chat whatsapp get "Contact" --before "2025-01-15T00:00:00Z"`
- **THEN** the CLI retrieves messages sent before that timestamp

#### Scenario: Time range filter

- **WHEN** user runs `chat whatsapp get "Contact" --since "7 days ago" --before "2 days ago"`
- **THEN** the CLI retrieves messages sent between 7 and 2 days ago

### Requirement: Watch Messages in Real-Time

The CLI MUST support streaming new messages as they arrive.

#### Scenario: Watch chat for new messages

- **WHEN** user runs `chat whatsapp watch "Contact"`
- **THEN** the CLI displays new messages as they arrive in real-time

#### Scenario: Watch all chats

- **WHEN** user runs `chat whatsapp watch --all`
- **THEN** the CLI displays new messages from all chats as they arrive

#### Scenario: Stop watching

- **WHEN** user presses Ctrl+C while watching
- **THEN** the CLI stops streaming and exits gracefully

### Requirement: Export Messages to File

The CLI MUST support exporting messages to various file formats.

#### Scenario: Export to JSON

- **WHEN** user runs `chat whatsapp export "Contact" --format json --output messages.json`
- **THEN** the CLI writes all messages to a JSON file

#### Scenario: Export to CSV

- **WHEN** user runs `chat whatsapp export "Contact" --format csv --output messages.csv`
- **THEN** the CLI writes messages to a CSV file with columns: timestamp, sender, content

#### Scenario: Export with filters

- **WHEN** user runs `chat whatsapp export "Contact" --since "30 days ago" --format json`
- **THEN** the CLI exports only messages from the last 30 days

### Requirement: Multiple Output Formats

The CLI MUST support multiple output formats for displaying messages.

#### Scenario: Text output format

- **WHEN** user runs `chat whatsapp get "Contact" --format text`
- **THEN** messages are displayed in human-readable text format

#### Scenario: JSON output format

- **WHEN** user runs `chat whatsapp get "Contact" --format json`
- **THEN** messages are displayed as JSON array

#### Scenario: Compact output format

- **WHEN** user runs `chat whatsapp get "Contact" --format compact`
- **THEN** messages are displayed in compact single-line format

### Requirement: Filter Messages by Sender

The CLI MUST support filtering messages by sender in group chats.

#### Scenario: Filter by sender phone

- **WHEN** user runs `chat whatsapp get "Group" --sender "+1234567890"`
- **THEN** the CLI displays only messages from that sender

#### Scenario: Filter by sender name

- **WHEN** user runs `chat whatsapp get "Group" --sender "John Doe"`
- **THEN** the CLI displays only messages from that sender

### Requirement: Filter Messages by Type

The CLI MUST support filtering messages by content type.

#### Scenario: Text messages only

- **WHEN** user runs `chat whatsapp get "Contact" --type text`
- **THEN** the CLI displays only text messages

#### Scenario: Media messages only

- **WHEN** user runs `chat whatsapp get "Contact" --type media`
- **THEN** the CLI displays only messages with images, videos, or files

#### Scenario: Multiple types

- **WHEN** user runs `chat whatsapp get "Contact" --type text,image`
- **THEN** the CLI displays text and image messages only

### Requirement: Configuration Management

The CLI MUST support managing configuration settings.

#### Scenario: Set configuration value

- **WHEN** user runs `chat whatsapp config set session_path "/custom/path"`
- **THEN** the CLI saves the session path to configuration file

#### Scenario: Get configuration value

- **WHEN** user runs `chat whatsapp config get session_path`
- **THEN** the CLI displays the stored session path

#### Scenario: List all configuration

- **WHEN** user runs `chat whatsapp config list`
- **THEN** the CLI displays all configuration settings

### Requirement: Search Messages

The CLI MUST support searching messages by text content.

#### Scenario: Search in specific chat

- **WHEN** user runs `chat whatsapp search "Contact" "search term"`
- **THEN** the CLI displays messages containing the search term

#### Scenario: Search all chats

- **WHEN** user runs `chat whatsapp search --all "search term"`
- **THEN** the CLI searches across all chats and displays matching messages

#### Scenario: Case-insensitive search

- **WHEN** user runs `chat whatsapp search "Contact" "term" --ignore-case`
- **THEN** the CLI performs case-insensitive search

### Requirement: Get Chat Information

The CLI MUST display detailed information about a specific chat.

#### Scenario: Get chat details

- **WHEN** user runs `chat whatsapp info "Contact Name"`
- **THEN** the CLI displays chat ID, type, phone number, and description

#### Scenario: Get group info

- **WHEN** user runs `chat whatsapp info "Group Name"`
- **THEN** the CLI displays group ID, member count, participants, and description

### Requirement: Download Media Files

The CLI MUST support downloading media files from messages.

#### Scenario: Download single media file

- **WHEN** user runs `chat whatsapp download --message-id "abc123" --output ./media/`
- **THEN** the CLI downloads the media file to the specified directory

#### Scenario: Download all media from chat

- **WHEN** user runs `chat whatsapp download "Contact" --all --output ./media/`
- **THEN** the CLI downloads all media files from the chat

#### Scenario: Download with filters

- **WHEN** user runs `chat whatsapp download "Contact" --type image --since "7 days ago"`
- **THEN** the CLI downloads only images from the last 7 days
