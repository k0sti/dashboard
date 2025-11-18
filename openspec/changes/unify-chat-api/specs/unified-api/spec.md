# Spec: Unified Chat API

**Capability:** `unified-api`
**Status:** Draft

## Overview

Defines the unified chat API with `ChatSource` trait and `SourcesManager` for accessing messages across different platforms (Telegram, Signal, WhatsApp) with a consistent interface.

## ADDED Requirements

### Requirement: ChatSource Trait

The system SHALL provide a `ChatSource` trait that abstracts over different chat platforms with consistent operations.

#### Scenario: List chats from Telegram source

**Given** a Telegram source is connected
**When** user calls `list_chats()` with no filter
**Then** system returns all Telegram chats with ID, name, type, and participant count

#### Scenario: Get messages with basic filter

**Given** a Telegram source is connected
**And** chat "Antti" exists
**When** user calls `get_messages()` with chat filter for "Antti"
**Then** system returns messages from that chat ordered by timestamp

#### Scenario: Subscribe to new messages

**Given** a Signal source is connected
**When** user calls `subscribe()`
**Then** system returns a receiver that streams new messages as they arrive

### Requirement: SourcesManager

The system SHALL provide a `SourcesManager` that registers and manages multiple chat sources.

#### Scenario: Register multiple sources

**Given** SourcesManager is initialized
**When** user registers Telegram, Signal, and WhatsApp sources
**Then** all three sources are available for querying
**And** each source has a unique identifier

#### Scenario: Query specific source

**Given** SourcesManager has Telegram and Signal registered
**When** user queries messages from "telegram" source
**Then** system only returns messages from Telegram
**And** Signal source is not queried

#### Scenario: Cross-source query

**Given** SourcesManager has multiple sources registered
**When** user queries messages without specifying source
**Then** system queries all registered sources
**And** returns combined results with source identifier

### Requirement: Source Identification

Each chat source SHALL have a unique string identifier and display name.

#### Scenario: Source has stable identifier

**Given** a Telegram source is registered
**Then** its source_id is "telegram"
**And** its source_name is "Telegram"
**And** the identifier remains constant across restarts

### Requirement: Connection Status

Each source SHALL report its connection status accurately.

#### Scenario: Check source connection

**Given** a Telegram source has established connection
**When** user checks `is_connected()`
**Then** system returns `true`

#### Scenario: Detect disconnection

**Given** a Signal source was connected but lost network
**When** user checks `is_connected()`
**Then** system returns `false`

### Requirement: Backward Compatibility

The system SHALL maintain the existing `ChatClient` trait for backward compatibility.

#### Scenario: Legacy ChatClient still works

**Given** code using the old `ChatClient` trait
**When** that code runs after migration
**Then** it continues to work without changes
**And** can coexist with new `ChatSource` code

## Dependencies

- Depends on existing Telegram, Signal, WhatsApp client implementations
- Requires async Rust (tokio)
- Needs serde for serialization

## Implementation Notes

- `ChatSource` trait should be object-safe (trait object compatible)
- `SourcesManager` should use interior mutability for dynamic registration
- Source identifiers should follow kebab-case convention (telegram, signal, whatsapp)
- Subscribe() returning None is acceptable for sources that don't support streaming
