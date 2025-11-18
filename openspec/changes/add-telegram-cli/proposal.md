# Change: Add Telegram CLI Binary

## Why

Users need a command-line interface to read and monitor Telegram messages programmatically. This enables scripting, automation, and integration with other tools without requiring a GUI.

## What Changes

- Add a new binary crate `chat-telegram` that provides a CLI for Telegram
- Implement commands: `init`, `status`, `list`, `get`, `watch`, `export`
- Support message filtering by time (`--since`, `--limit`, `--before`, `--after`)
- Support multiple output formats (text, JSON, CSV)
- Implement session management and authentication

## Impact

- Affected specs: telegram-cli (new capability)
- Affected code:
  - New crate: `crates/chat-telegram/`
  - Uses: `crates/chat/` library
  - Binary: `chat-telegram` or `chat telegram` subcommand
