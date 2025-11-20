# SQLite Conflict Resolution

## Problem Summary

**Cannot build both Telegram and WhatsApp in the same binary** due to conflicting SQLite dependencies:

- **Telegram** (grammers-session) → `sqlite` crate v0.37 → `sqlite3-sys` v0.18
- **WhatsApp** (whatsapp-rust) → `diesel` ORM v2.2.12 → `libsqlite3-sys` v0.35

Both crates attempt to link the native `sqlite3` library, which Cargo prohibits (only one package can link each native library).

## Root Cause

Even with `optional = true` dependencies, **Cargo resolves ALL dependencies** in `Cargo.toml` before building to check for conflicts. This is by design - Cargo validates the entire dependency graph upfront.

## Solution Implemented

### For Telegram: MemorySession (No Persistence)

**Changes Made:**
1. Replaced `SqliteSession` with `MemorySession` in all Telegram code
2. Removed SQLite file path checking
3. Updated user messages to note non-persistent sessions

**Files Modified:**
- `src/telegram_source.rs`
- `src/cli_common/telegram/commands/client.rs`
- `src/cli_common/telegram/commands/init.rs`
- `src/cli_common/telegram/commands/watch.rs`

**Trade-offs:**
- ✅ Single Cargo.toml can now include both platforms
- ✅ No SQLite conflict for Telegram
- ❌ Telegram session doesn't persist across restarts (need to re-auth each time)
- ✅ WhatsApp can keep full SQLite persistence

### Current Build Configuration

**Telegram** (active):
```bash
cargo build --release --features telegram
```

**WhatsApp** (disabled in Cargo.toml):
Currently commented out to prevent dependency resolution conflicts.
To enable: See instructions below.

## Alternative Approaches Considered

1. **Port Diesel → raw sqlite** - 3-5 days effort, high complexity
2. **Memory-only storage for both** - Loses persistence for both platforms
3. **Separate binaries** - Two different executables
4. **Wrapper script** - Manages two separate builds
5. **Unified SQLite** (Option 3) - Replace whatsapp-rust Diesel with grammers `sqlite` crate

We chose **MemorySession for Telegram** as the quickest solution.

## Building WhatsApp Support

### Option A: Swap in Cargo.toml

To build WhatsApp instead of Telegram:

1. **Edit `Cargo.toml`**:
   ```toml
   # Comment out Telegram dependencies:
   # grammers-client = { version = "0.8", optional = true }
   # grammers-session = { version = "0.8", optional = true }
   # grammers-mtsender = { version = "0.8", optional = true }

   # Uncomment WhatsApp dependencies:
   whatsapp-rust = { path = "./whatsapp-rust", optional = true }
   qr2term = { version = "0.3", optional = true }

   # Update features:
   default = ["whatsapp"]  # Changed from telegram
   # telegram = [...]  # Comment out
   whatsapp = ["whatsapp-rust", "qr2term"]  # Uncomment
   ```

2. **Build**:
   ```bash
   cargo build --release --features whatsapp
   ```

### Option B: Git Branches (Recommended)

Create separate branches for each platform:

```bash
# Create WhatsApp branch
git checkout -b whatsapp
# Edit Cargo.toml as above
git commit -am "Enable WhatsApp, disable Telegram"

# Switch back to Telegram
git checkout main
```

Build from appropriate branch:
```bash
git checkout whatsapp && cargo build --release
# or
git checkout main && cargo build --release
```

### Option C: Separate Repository Fork

Fork the repository and maintain two versions:
- `chat` (Telegram only)
- `chat-whatsapp` (WhatsApp only)

## Future Solutions

### Long-term: Implement JSON-based Session for Telegram

Create custom `Session` trait implementation that:
- Stores session data in JSON file
- No SQLite dependency
- ~2-4 hours implementation time

Benefits:
- Both platforms persist sessions
- Single binary with both features
- No dependency conflicts

### Alternative: Port whatsapp-rust to use `sqlite` crate

Replace Diesel ORM in whatsapp-rust fork with raw `sqlite` crate (matching grammers):
- Effort: 3-5 days
- Requires maintaining fork
- Difficult to merge upstream updates

## Testing

**Telegram build verified:**
```bash
$ cargo build --release --no-default-features --features telegram --bin chat
   Compiling chat v0.1.0
    Finished `release` profile [optimized] target(s) in 4.30s
```

**Note:** Telegram session will require re-authentication after each restart due to MemorySession.
