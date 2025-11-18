# Spec: Message Filters

**Capability:** `message-filters`
**Status:** Draft

## Overview

Defines structured filters for querying messages and chats across different platforms with consistent syntax and semantics.

## ADDED Requirements

### Requirement: MessageFilter Structure

The system SHALL provide a `MessageFilter` struct that supports filtering messages by chat, time range, sender, content, and type.

#### Scenario: Filter by chat name pattern

**Given** a Telegram source with chats "Antti", "Anna", "Bob"
**When** user creates filter with chat pattern `Name("Ann")`
**Then** system matches chats "Antti" and "Anna"
**And** does not match "Bob"

#### Scenario: Filter by time range

**Given** messages exist from January 1-31, 2025
**When** user sets `since = 2025-01-15` and `before = 2025-01-20`
**Then** system returns only messages from January 15-19
**And** excludes messages from January 14 and earlier
**And** excludes messages from January 20 and later

#### Scenario: Filter by sender

**Given** messages from senders "Alice", "Bob", "Charlie"
**When** user sets `sender = Some("Alice")`
**Then** system returns only messages from Alice
**And** excludes messages from Bob and Charlie

#### Scenario: Filter by text search

**Given** messages contain "meeting", "project", "deadline"
**When** user sets `search = Some("meeting")`
**Then** system returns messages containing "meeting"
**And** performs case-insensitive substring match

#### Scenario: Limit results

**Given** a chat has 1000 messages
**When** user sets `limit = Some(10)`
**Then** system returns exactly 10 messages
**And** returns the most recent messages first

### Requirement: ChatPattern Matching

The system SHALL support multiple chat matching patterns.

#### Scenario: Match by specific ID

**Given** chats with IDs "123", "456", "789"
**When** user uses pattern `ChatPattern::Id(ChatId::new("456"))`
**Then** system matches only chat "456"

#### Scenario: Match all chats

**Given** a source with multiple chats
**When** user uses pattern `ChatPattern::All`
**Then** system matches all chats in the source

#### Scenario: Match multiple specific chats

**Given** chats with IDs "123", "456", "789"
**When** user uses pattern `ChatPattern::Multiple(vec!["123", "789"])`
**Then** system matches chats "123" and "789"
**And** excludes chat "456"

### Requirement: ChatFilter Structure

The system SHALL provide a `ChatFilter` struct for filtering chat lists.

#### Scenario: Filter by chat type

**Given** chats of types Direct, Group, Channel
**When** user sets `chat_type = Some(ChatType::Group)`
**Then** system returns only Group chats
**And** excludes Direct and Channel chats

#### Scenario: Filter by name pattern

**Given** chats named "Work", "Family", "Friends"
**When** user sets `name_pattern = Some("Fam")`
**Then** system returns "Family" chat
**And** performs case-insensitive substring match

#### Scenario: Filter by recent activity

**Given** chats with last activity on different dates
**When** user sets `active_since = Some(7 days ago)`
**Then** system returns only chats active in last 7 days
**And** excludes older chats

### Requirement: Content Type Filtering

The system SHALL support filtering messages by content type.

#### Scenario: Filter text messages only

**Given** messages with types Text, Image, Video
**When** user sets `content_type = Some(vec![ContentType::Text])`
**Then** system returns only text messages
**And** excludes Image and Video messages

#### Scenario: Filter multiple content types

**Given** messages with types Text, Image, Video, Audio
**When** user sets `content_type = Some(vec![ContentType::Image, ContentType::Video])`
**Then** system returns Image and Video messages
**And** excludes Text and Audio messages

### Requirement: Filter Composition

Multiple filter criteria SHALL be combined with AND logic.

#### Scenario: Combine chat, time, and sender filters

**Given** messages in multiple chats from different senders
**When** user sets:
- `chat = ChatPattern::Name("Antti")`
- `since = Some(7 days ago)`
- `sender = Some("Alice")`
**Then** system returns messages from Alice in "Antti" chat from last 7 days
**And** excludes messages not matching all three criteria

### Requirement: Filter Validation

The system SHALL validate filters before execution.

#### Scenario: Reject invalid time range

**Given** user creates filter
**When** user sets `since = 2025-01-20` and `before = 2025-01-10`
**Then** system returns validation error
**And** error indicates "since must be before before"

#### Scenario: Reject negative limit

**Given** user creates filter
**When** user sets `limit = Some(0)`
**Then** system returns validation error
**And** error indicates "limit must be positive"

## Dependencies

- Depends on core types (Message, Chat, ChatId)
- Requires chrono for DateTime handling
- Needs regex support for pattern matching

## Implementation Notes

- ChatPattern::Name should support partial matching (case-insensitive substring)
- Time filtering should handle timezone conversions
- Search should be case-insensitive by default
- Limit should apply after all other filters
- Empty filters (all None) should return all results
- Filter validation should happen at construction time
