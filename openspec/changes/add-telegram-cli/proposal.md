# Change: Add Telegram CLI Binary

## Why

Users need a command-line interface to read and monitor Telegram messages programmatically. This enables scripting, automation, and integration with other tools without requiring a GUI.

## What Changes

- Add Telegram CLI to the unified `chat` binary with `chat telegram` subcommands
- Implement commands: `init`, `status`, `list`, `get`, `watch`, `export`, `search`, `info`, `config`, `logout`
- Support message filtering by time (`--since`, `--limit`, `--before`, `--after`)
- Support multiple output formats (text, JSON, CSV, compact)
- Implement session management and authentication

## Impact

- Affected specs: telegram-cli (new capability)
- Affected code:
  - Modified crate: `crates/chat/`
  - New modules: `crates/chat/src/cli_common/telegram/` and `crates/chat/src/bin/main.rs`
  - Binary: `chat telegram <command>` (unified CLI)
