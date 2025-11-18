# Change: Add Signal CLI Binary

## Why

Users need a command-line interface to read and monitor Signal messages programmatically. This enables scripting, automation, and integration with other tools while maintaining Signal's privacy focus.

## What Changes

- Add a new binary crate `chat-signal` that provides a CLI for Signal
- Implement commands: `init`, `status`, `list`, `get`, `watch`, `export`
- Support message filtering by time (`--since`, `--limit`, `--before`, `--after`)
- Support multiple output formats (text, JSON, CSV)
- Implement phone number registration and verification
- **Note:** Signal's libsignal is designed for Signal's own use; community alternatives may be needed

## Impact

- Affected specs: signal-cli (new capability)
- Affected code:
  - New crate: `crates/chat-signal/`
  - Uses: `crates/chat/` library
  - Binary: `chat-signal` or `chat signal` subcommand
- **Note:** May require linking with Signal's primary device via QR code
