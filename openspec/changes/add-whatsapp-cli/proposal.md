# Change: Add WhatsApp CLI Binary

## Why

Users need a command-line interface to read and monitor WhatsApp messages programmatically. This enables scripting, automation, and integration with other tools without requiring a GUI or web interface.

## What Changes

- Add a new binary crate `chat-whatsapp` that provides a CLI for WhatsApp
- Implement commands: `init`, `status`, `list`, `get`, `watch`, `export`
- Support message filtering by time (`--since`, `--limit`, `--before`, `--after`)
- Support multiple output formats (text, JSON, CSV)
- Implement QR code authentication for WhatsApp Web protocol
- **Note:** WhatsApp uses unofficial client libraries which may violate Terms of Service

## Impact

- Affected specs: whatsapp-cli (new capability)
- Affected code:
  - New crate: `crates/chat-whatsapp/`
  - Uses: `crates/chat/` library
  - Binary: `chat-whatsapp` or `chat whatsapp` subcommand
- **Risk:** Unofficial WhatsApp client usage may result in account restrictions
