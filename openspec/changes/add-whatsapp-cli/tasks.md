# Implementation Tasks: WhatsApp CLI

## 1. Project Setup

- [ ] 1.1 Create `crates/chat-whatsapp/` binary crate
- [ ] 1.2 Add dependencies: `clap`, `tokio`, `anyhow`, `chrono`, `serde`, `serde_json`
- [ ] 1.3 Add dependency on `chat` crate
- [ ] 1.4 Set up binary target in Cargo.toml
- [ ] 1.5 Create basic CLI structure with clap
- [ ] 1.6 Add QR code display library (`qr2term` or similar)

## 2. Core Commands

- [ ] 2.1 Implement `init` command with QR code authentication
- [ ] 2.2 Implement `status` command to show connection state
- [ ] 2.3 Implement `list` command to enumerate chats
- [ ] 2.4 Implement `get` command to retrieve messages
- [ ] 2.5 Implement `watch` command for real-time message streaming
- [ ] 2.6 Implement `export` command for message export

## 3. Filtering Options

- [ ] 3.1 Add `--limit` flag for message count limit
- [ ] 3.2 Add `--since` flag for time-based filtering (absolute and relative)
- [ ] 3.3 Add `--before` flag for upper time bound
- [ ] 3.4 Add `--after` flag (alias for `--since`)
- [ ] 3.5 Add `--sender` flag for filtering by sender
- [ ] 3.6 Add `--type` flag for filtering by message type

## 4. Output Formats

- [ ] 4.1 Implement text output formatter
- [ ] 4.2 Implement JSON output formatter
- [ ] 4.3 Implement CSV output formatter
- [ ] 4.4 Implement compact output formatter
- [ ] 4.5 Add `--format` flag to all relevant commands
- [ ] 4.6 Add `--output` flag to write to file

## 5. Configuration Management

- [ ] 5.1 Implement config file storage (TOML or JSON)
- [ ] 5.2 Implement `config set` subcommand
- [ ] 5.3 Implement `config get` subcommand
- [ ] 5.4 Implement `config list` subcommand
- [ ] 5.5 Support environment variable overrides
- [ ] 5.6 Document session file location and format

## 6. Additional Commands

- [ ] 6.1 Implement `search` command for text search
- [ ] 6.2 Implement `info` command for chat details
- [ ] 6.3 Implement `download` command for media files
- [ ] 6.4 Implement `logout` command to clear session

## 7. WhatsApp-Specific Features

- [ ] 7.1 Add QR code regeneration on timeout
- [ ] 7.2 Handle phone number formatting for contacts
- [ ] 7.3 Support WhatsApp group chat IDs (@g.us)
- [ ] 7.4 Support individual chat IDs (@c.us)
- [ ] 7.5 Handle WhatsApp Business accounts

## 8. Error Handling & UX

- [ ] 8.1 Add comprehensive error messages
- [ ] 8.2 Add progress indicators for long operations
- [ ] 8.3 Add colored output support (optional via flag)
- [ ] 8.4 Add quiet/verbose modes
- [ ] 8.5 Handle Ctrl+C gracefully in all commands
- [ ] 8.6 Add shell completion generation
- [ ] 8.7 Add warnings about ToS violations

## 9. Testing

- [ ] 9.1 Add unit tests for command parsing
- [ ] 9.2 Add unit tests for formatters
- [ ] 9.3 Add integration tests with mock client
- [ ] 9.4 Add manual testing guide
- [ ] 9.5 Test with real WhatsApp account (use test account)

## 10. Documentation

- [ ] 10.1 Add README.md with usage examples
- [ ] 10.2 Add INSTALLATION.md with setup instructions
- [ ] 10.3 Document all commands with `--help`
- [ ] 10.4 Add troubleshooting guide
- [ ] 10.5 Add configuration file format documentation
- [ ] 10.6 Add disclaimer about unofficial client risks

## 11. Build & Distribution

- [ ] 11.1 Configure release builds
- [ ] 11.2 Add installation script
- [ ] 11.3 Test on Linux, macOS, and Windows
- [ ] 11.4 Create binary releases (optional)
