# Single Binary Investigation: Telegram + WhatsApp

## Current Situation

**Goal**: Support both Telegram and WhatsApp in a single `chat` binary.

**Blocker**: SQLite dependency conflict
- `grammers-session` v0.8.0 → `sqlite` v0.37 → `sqlite3-sys` v0.18
- `whatsapp-rust` → `diesel` v2.2.12 → `libsqlite3-sys` v0.35

**Root Cause**: Even though we use `MemorySession` (which doesn't need SQLite), `grammers-session` declares `sqlite` as a **non-optional** dependency in its Cargo.toml.

## Investigation Results

### Verified Facts

1. ✅ `MemorySession` implementation doesn't use sqlite (checked source)
2. ✅ Telegram works fine with MemorySession (tested)
3. ❌ `grammers-session` Cargo.toml has `[dependencies.sqlite]` (not optional)
4. ❌ Cargo resolves ALL dependencies even if we don't use them
5. ❌ Cannot have both dependencies in same Cargo.toml (native library link conflict)

## Possible Solutions

### Option 1: Patch grammers-session to Make SQLite Optional ⭐ RECOMMENDED

**Approach**: Use Cargo's `[patch]` feature to override grammers-session

**Implementation**:
1. Fork grammers or create local patch
2. Make sqlite dependency optional
3. Use Cargo patch in our Cargo.toml

**Cargo.toml**:
```toml
[patch.crates-io]
grammers-session = { path = "./patches/grammers-session" }
```

**Effort**: 1-2 hours
**Pros**: Clean solution, both features work, single binary
**Cons**: Requires maintaining patch, might break on updates

### Option 2: Use Different Telegram Library

**Approach**: Replace grammers with a library that doesn't use SQLite

**Options**:
- `tdlib` (Telegram's official library) - C++ with Rust bindings
- `teloxide` - Higher-level, uses tdlib
- `frankenstein` - Telegram Bot API only

**Effort**: Several days (major refactor)
**Pros**: Might avoid SQLite conflict
**Cons**: Major code changes, may have other limitations

### Option 3: Dynamic Loading (dlopen)

**Approach**: Load one of the platforms as a dynamic library

**Implementation**:
- Build whatsapp-rust as separate .so/.dylib
- Use `libloading` to dynamically load at runtime
- Only one SQLite version linked into main binary

**Effort**: 4-6 hours
**Pros**: Single binary artifact
**Cons**: Complex, platform-specific, harder debugging

### Option 4: Separate Binaries with Wrapper ⭐ FALLBACK

**Approach**: Build two binaries, create smart wrapper

**Implementation**:
```bash
# Two separate binaries
chat-telegram (with telegram feature)
chat-whatsapp (with whatsapp feature)

# Wrapper script
chat telegram list  → calls chat-telegram list
chat whatsapp get   → calls chat-whatsapp get
```

**Effort**: 2-3 hours
**Pros**: Simple, reliable, no hacks
**Cons**: Two binaries to distribute

### Option 5: Conditional Compilation with Mutually Exclusive Features

**Approach**: Use Cargo features that are mutually exclusive

**Cargo.toml**:
```toml
[features]
default = ["telegram"]
telegram = ["grammers-client", "grammers-session", "grammers-mtsender"]
whatsapp = ["whatsapp-rust", "qr2term"]

# Note: Cannot enable both at once
```

**Build commands**:
```bash
cargo build --features telegram      # Telegram binary
cargo build --features whatsapp      # WhatsApp binary
```

**Effort**: Already done!
**Pros**: Simple, works now
**Cons**: Cannot have both in one binary, must choose at compile time

## Recommendation

### For Immediate Use (Today)

**Use Option 5** (already implemented):
- Build with `--features telegram` for Telegram
- Build with `--features whatsapp` for WhatsApp
- User picks which platform at build time

### For Single Binary (Best Long-term Solution)

**Implement Option 1** (Patch grammers-session):

1. Create local patch of grammers-session with optional SQLite
2. Use Cargo [patch] to override dependency
3. Both platforms can coexist in single binary

**Estimated time**: 1-2 hours

### Alternative if Patch Fails

**Fall back to Option 4** (Separate binaries + wrapper):
- Build two binaries
- Create shell wrapper for unified CLI
- Still better UX than manual switching

## Next Steps

To proceed with **Option 1 (Patch)**:

1. Clone grammers repository or create minimal patch
2. Edit grammers-session/Cargo.toml to make sqlite optional
3. Add conditional compilation in sqlite.rs
4. Use [patch.crates-io] in our Cargo.toml
5. Test build with both features enabled

**Would you like me to implement Option 1?**
