# Spec: CLI Commands

**Capability:** `cli-commands`
**Status:** Draft

## Overview

Defines the command-line interface for accessing chat operations with unified syntax across all sources.

## ADDED Requirements

### Requirement: Sources Command

The system SHALL provide a `sources` command to list all configured chat sources.

#### Scenario: List all sources

**Given** Telegram, Signal, and WhatsApp are configured
**When** user runs `chat sources`
**Then** system displays all three sources
**And** shows source ID, name, and connection status for each

#### Scenario: Show disconnected source

**Given** Telegram is connected but Signal is not
**When** user runs `chat sources`
**Then** Telegram shows status "Connected"
**And** Signal shows status "Disconnected"

### Requirement: Chats Command

The system SHALL provide a `chats` command to list conversations from a source.

#### Scenario: List all chats from source

**Given** Telegram source has 10 chats
**When** user runs `chat chats telegram`
**Then** system displays all 10 chats
**And** shows chat ID, name, type, and participant count for each

#### Scenario: Filter chats by name pattern

**Given** Telegram has chats "Work", "Family", "Friends"
**When** user runs `chat chats telegram --name="Fam"`
**Then** system displays only "Family" chat

#### Scenario: Filter chats by type

**Given** Telegram has Direct, Group, and Channel chats
**When** user runs `chat chats telegram --type=group`
**Then** system displays only Group chats

### Requirement: Groups Command

The system SHALL provide a `groups` command to list only group chats.

#### Scenario: List groups from source

**Given** Telegram has 3 Direct chats and 2 Groups
**When** user runs `chat groups telegram`
**Then** system displays only the 2 Groups
**And** excludes Direct chats

### Requirement: Messages Command

The system SHALL provide a `messages` command with filter syntax.

#### Scenario: Get messages from specific chat by name

**Given** Telegram has chat named "Antti"
**When** user runs `chat messages telegram:Antti`
**Then** system returns messages from "Antti" chat
**And** displays most recent messages first

#### Scenario: Get messages from chat by ID

**Given** Telegram has chat with ID "123456"
**When** user runs `chat messages telegram:123456`
**Then** system returns messages from that chat

#### Scenario: Get messages with time filter

**Given** chat "Antti" has messages from last 30 days
**When** user runs `chat messages telegram:Antti --since=7d`
**Then** system returns only messages from last 7 days

#### Scenario: Get messages with sender filter

**Given** chat "Antti" has messages from Alice and Bob
**When** user runs `chat messages telegram:Antti --sender=Alice`
**Then** system returns only messages from Alice

#### Scenario: Get messages with text search

**Given** chat "Antti" has various messages
**When** user runs `chat messages telegram:Antti --search="meeting"`
**Then** system returns only messages containing "meeting"
**And** search is case-insensitive

#### Scenario: Limit number of messages

**Given** chat "Antti" has 1000 messages
**When** user runs `chat messages telegram:Antti --limit=10`
**Then** system returns exactly 10 most recent messages

#### Scenario: Combine multiple filters

**Given** chat "Antti" has many messages
**When** user runs `chat messages telegram:Antti --since=7d --sender=Alice --search="project" --limit=5`
**Then** system returns up to 5 messages from Alice containing "project" from last 7 days

#### Scenario: Search across all chats in source

**Given** Telegram has multiple chats
**When** user runs `chat messages telegram:* --search="meeting"`
**Then** system searches all Telegram chats
**And** returns matching messages with chat identification

#### Scenario: Query across all sources

**Given** Telegram and Signal are configured
**When** user runs `chat messages "*:*" --since=1d`
**Then** system queries both sources
**And** returns combined results with source identification

### Requirement: Output Formats

All commands SHALL support multiple output formats.

#### Scenario: JSON output format

**Given** user requests messages
**When** user runs `chat messages telegram:Antti --format=json`
**Then** system outputs valid JSON array
**And** each message includes all fields (id, sender, content, timestamp)

#### Scenario: CSV output format

**Given** user requests messages
**When** user runs `chat messages telegram:Antti --format=csv`
**Then** system outputs CSV with headers
**And** each row contains message data

#### Scenario: Compact text format

**Given** user requests messages
**When** user runs `chat messages telegram:Antti --format=compact`
**Then** system outputs one message per line
**And** format is: `[timestamp] sender: content`

#### Scenario: Default text format

**Given** user requests messages
**When** user runs `chat messages telegram:Antti` (no format specified)
**Then** system outputs human-readable text format
**And** uses colors and formatting for readability

### Requirement: Time Specification

Time filters SHALL support multiple formats.

#### Scenario: Relative time (days)

**Given** current date is 2025-01-20
**When** user specifies `--since=7d`
**Then** system interprets as 2025-01-13

#### Scenario: Relative time (hours)

**Given** current time is 2025-01-20 14:00:00
**When** user specifies `--since=2h`
**Then** system interprets as 2025-01-20 12:00:00

#### Scenario: Absolute ISO date

**Given** user specifies `--since=2025-01-15`
**Then** system interprets as 2025-01-15 00:00:00 UTC

#### Scenario: Absolute ISO datetime

**Given** user specifies `--since="2025-01-15T14:30:00Z"`
**Then** system interprets as 2025-01-15 14:30:00 UTC

### Requirement: Error Handling

Commands SHALL provide helpful error messages.

#### Scenario: Source not found

**Given** only Telegram is configured
**When** user runs `chat chats signal`
**Then** system displays error: "Source 'signal' not found"
**And** suggests: "Run 'chat sources' to see available sources"

#### Scenario: Chat not found

**Given** Telegram has no chat named "Invalid"
**When** user runs `chat messages telegram:Invalid`
**Then** system displays error: "Chat 'Invalid' not found"
**And** suggests: "Run 'chat chats telegram' to see available chats"

#### Scenario: Source not connected

**Given** Signal is configured but disconnected
**When** user runs `chat chats signal`
**Then** system displays error: "Source 'signal' is not connected"
**And** provides connection instructions

### Requirement: Help and Documentation

Each command SHALL provide helpful usage information.

#### Scenario: Command help

**Given** user needs help with messages command
**When** user runs `chat messages --help`
**Then** system displays usage syntax
**And** shows all available options
**And** provides examples

#### Scenario: No arguments

**Given** user runs `chat` with no subcommand
**Then** system displays available commands
**And** shows brief description for each

## Dependencies

- Depends on `unified-api` for ChatSource interface
- Depends on `message-filters` for filter parsing
- Requires clap for CLI parsing
- Needs chrono for time parsing

## Implementation Notes

- Filter syntax: `source:chat_pattern`
- Wildcard `*` matches all (sources or chats)
- Time units: s (seconds), m (minutes), h (hours), d (days), w (weeks)
- Default format: human-readable text with colors
- Default limit: 100 messages
- Chat pattern matching should be case-insensitive
- Source IDs should follow kebab-case (telegram, signal, whatsapp)
