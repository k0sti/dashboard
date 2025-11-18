# Signal CLI Specification

## ADDED Requirements

### Requirement: Initialize Signal Connection

The CLI MUST support initializing and registering a Signal account with phone verification.

#### Scenario: First-time registration

- **WHEN** user runs `chat signal init` with phone number
- **THEN** the CLI sends verification SMS
- **AND** prompts for verification code
- **AND** saves account credentials for future use

#### Scenario: Link as secondary device

- **WHEN** user runs `chat signal init --link`
- **THEN** the CLI displays a QR code to link with primary Signal device

#### Scenario: Resume existing session

- **WHEN** user runs `chat signal init` and credentials exist
- **THEN** the CLI uses existing session without re-registration

### Requirement: Check Connection Status

The CLI MUST allow users to check the current connection status.

#### Scenario: Connected status

- **WHEN** user runs `chat signal status`
- **THEN** the CLI displays connection state, phone number, and registration date

#### Scenario: Disconnected status

- **WHEN** user runs `chat signal status` while disconnected
- **THEN** the CLI indicates not connected and suggests running `init`

### Requirement: List All Chats

The CLI MUST list all available conversations and groups.

#### Scenario: List all chats

- **WHEN** user runs `chat signal list`
- **THEN** the CLI displays all chats with ID, name, and type (individual/group)

#### Scenario: List with JSON output

- **WHEN** user runs `chat signal list --format json`
- **THEN** the CLI outputs chat list in JSON format

#### Scenario: Filter by chat type

- **WHEN** user runs `chat signal list --type group`
- **THEN** the CLI displays only group chats

### Requirement: Retrieve Messages from Chat

The CLI MUST retrieve messages from a specified chat by contact name or ID.

#### Scenario: Get recent messages by contact name

- **WHEN** user runs `chat signal get "Contact Name"`
- **THEN** the CLI displays recent messages from that chat

#### Scenario: Get messages by phone number

- **WHEN** user runs `chat signal get "+1234567890"`
- **THEN** the CLI displays messages from that contact

#### Scenario: Get messages by UUID

- **WHEN** user runs `chat signal get --uuid "550e8400-e29b-41d4-a716-446655440000"`
- **THEN** the CLI displays messages from the contact with that UUID

#### Scenario: Chat not found

- **WHEN** user runs `chat signal get "Nonexistent Chat"`
- **THEN** the CLI displays an error indicating chat not found

### Requirement: Limit Messages by Count

The CLI MUST support limiting the number of messages retrieved.

#### Scenario: Limit message count

- **WHEN** user runs `chat signal get "Contact" --limit 50`
- **THEN** the CLI retrieves at most 50 messages, starting from the most recent

#### Scenario: Default limit

- **WHEN** user runs `chat signal get "Contact"` without `--limit`
- **THEN** the CLI retrieves at most 100 messages by default

### Requirement: Filter Messages by Time

The CLI MUST support filtering messages by timestamp or relative time.

#### Scenario: Messages since absolute timestamp

- **WHEN** user runs `chat signal get "Contact" --since "2025-01-01T00:00:00Z"`
- **THEN** the CLI retrieves messages sent after that timestamp

#### Scenario: Messages since relative time

- **WHEN** user runs `chat signal get "Contact" --since "2 days ago"`
- **THEN** the CLI retrieves messages sent in the last 2 days

#### Scenario: Messages before timestamp

- **WHEN** user runs `chat signal get "Contact" --before "2025-01-15T00:00:00Z"`
- **THEN** the CLI retrieves messages sent before that timestamp

#### Scenario: Time range filter

- **WHEN** user runs `chat signal get "Contact" --since "7 days ago" --before "2 days ago"`
- **THEN** the CLI retrieves messages sent between 7 and 2 days ago

### Requirement: Watch Messages in Real-Time

The CLI MUST support streaming new messages as they arrive.

#### Scenario: Watch chat for new messages

- **WHEN** user runs `chat signal watch "Contact"`
- **THEN** the CLI displays new messages as they arrive in real-time

#### Scenario: Watch all chats

- **WHEN** user runs `chat signal watch --all`
- **THEN** the CLI displays new messages from all chats as they arrive

#### Scenario: Stop watching

- **WHEN** user presses Ctrl+C while watching
- **THEN** the CLI stops streaming and exits gracefully

### Requirement: Export Messages to File

The CLI MUST support exporting messages to various file formats.

#### Scenario: Export to JSON

- **WHEN** user runs `chat signal export "Contact" --format json --output messages.json`
- **THEN** the CLI writes all messages to a JSON file

#### Scenario: Export to CSV

- **WHEN** user runs `chat signal export "Contact" --format csv --output messages.csv`
- **THEN** the CLI writes messages to a CSV file with columns: timestamp, sender, content

#### Scenario: Export with filters

- **WHEN** user runs `chat signal export "Contact" --since "30 days ago" --format json`
- **THEN** the CLI exports only messages from the last 30 days

### Requirement: Multiple Output Formats

The CLI MUST support multiple output formats for displaying messages.

#### Scenario: Text output format

- **WHEN** user runs `chat signal get "Contact" --format text`
- **THEN** messages are displayed in human-readable text format

#### Scenario: JSON output format

- **WHEN** user runs `chat signal get "Contact" --format json`
- **THEN** messages are displayed as JSON array

#### Scenario: Compact output format

- **WHEN** user runs `chat signal get "Contact" --format compact`
- **THEN** messages are displayed in compact single-line format

### Requirement: Filter Messages by Sender

The CLI MUST support filtering messages by sender in group chats.

#### Scenario: Filter by sender phone

- **WHEN** user runs `chat signal get "Group" --sender "+1234567890"`
- **THEN** the CLI displays only messages from that sender

#### Scenario: Filter by sender UUID

- **WHEN** user runs `chat signal get "Group" --sender-uuid "550e8400-e29b-41d4-a716-446655440000"`
- **THEN** the CLI displays only messages from that sender

### Requirement: Filter Messages by Type

The CLI MUST support filtering messages by content type.

#### Scenario: Text messages only

- **WHEN** user runs `chat signal get "Contact" --type text`
- **THEN** the CLI displays only text messages

#### Scenario: Media messages only

- **WHEN** user runs `chat signal get "Contact" --type media`
- **THEN** the CLI displays only messages with images, videos, or files

#### Scenario: Multiple types

- **WHEN** user runs `chat signal get "Contact" --type text,image`
- **THEN** the CLI displays text and image messages only

### Requirement: Configuration Management

The CLI MUST support managing configuration settings.

#### Scenario: Set configuration value

- **WHEN** user runs `chat signal config set data_path "/custom/path"`
- **THEN** the CLI saves the data path to configuration file

#### Scenario: Get configuration value

- **WHEN** user runs `chat signal config get data_path`
- **THEN** the CLI displays the stored data path

#### Scenario: List all configuration

- **WHEN** user runs `chat signal config list`
- **THEN** the CLI displays all configuration settings

### Requirement: Search Messages

The CLI MUST support searching messages by text content.

#### Scenario: Search in specific chat

- **WHEN** user runs `chat signal search "Contact" "search term"`
- **THEN** the CLI displays messages containing the search term

#### Scenario: Search all chats

- **WHEN** user runs `chat signal search --all "search term"`
- **THEN** the CLI searches across all chats and displays matching messages

#### Scenario: Case-insensitive search

- **WHEN** user runs `chat signal search "Contact" "term" --ignore-case`
- **THEN** the CLI performs case-insensitive search

### Requirement: Get Chat Information

The CLI MUST display detailed information about a specific chat.

#### Scenario: Get contact details

- **WHEN** user runs `chat signal info "Contact Name"`
- **THEN** the CLI displays contact UUID, phone number, and profile information

#### Scenario: Get group info

- **WHEN** user runs `chat signal info "Group Name"`
- **THEN** the CLI displays group ID, member count, participants, and description

### Requirement: Download Media Files

The CLI MUST support downloading media files from messages.

#### Scenario: Download single media file

- **WHEN** user runs `chat signal download --message-id "abc123" --output ./media/`
- **THEN** the CLI downloads the media file to the specified directory

#### Scenario: Download all media from chat

- **WHEN** user runs `chat signal download "Contact" --all --output ./media/`
- **THEN** the CLI downloads all media files from the chat

#### Scenario: Download with filters

- **WHEN** user runs `chat signal download "Contact" --type image --since "7 days ago"`
- **THEN** the CLI downloads only images from the last 7 days

### Requirement: Verify Safety Numbers

The CLI MUST support verifying Signal safety numbers for contacts.

#### Scenario: Display safety number

- **WHEN** user runs `chat signal verify "Contact"`
- **THEN** the CLI displays the safety number for that contact

#### Scenario: Mark contact as verified

- **WHEN** user runs `chat signal verify "Contact" --trust`
- **THEN** the CLI marks the contact as verified

#### Scenario: Safety number changed

- **WHEN** safety number changes for a contact
- **THEN** the CLI displays a warning when retrieving messages from that contact
