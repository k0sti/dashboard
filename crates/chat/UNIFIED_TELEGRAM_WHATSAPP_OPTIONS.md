# Options for Single Program Supporting Both Telegram and WhatsApp

**Problem**: SQLite dependency conflict prevents building with both `telegram` and `whatsapp` features simultaneously.

**Goal**: Single `chat` binary that can access both Telegram and WhatsApp messages.

## Options Ranked by Feasibility

### ⭐ Option 1: Alternative WhatsApp Library (RECOMMENDED)

**Use a different WhatsApp library that doesn't use SQLite.**

#### Candidates:

1. **whatsappweb-rs** / **whatsappweb-eta**
   - Older library (last updated ~6 years ago)
   - May not use SQLite for session storage
   - **Action**: Test if it compiles with grammers

2. **Build minimal WhatsApp client yourself**
   - Use only protocol libraries (protobuf, encryption)
   - Implement custom session storage using same SQLite binding as grammers
   - Most work, but full control

3. **whatsapp-cloud-api** (Business API)
   - Official WhatsApp Business Cloud API
   - No unofficial client risk
   - ❌ **Limitation**: Only works with Business accounts, not personal groups
   - ❌ **Limitation**: Requires Business API setup

#### Implementation:
```toml
# Replace in Cargo.toml:
# whatsapp-rust = { version = "0.1.0-alpha", optional = true }

# Try option 1:
whatsappweb-eta = { version = "0.1", optional = true }  # If compatible
# OR Build minimal client with:
prost = "0.12"  # Protobuf
curve25519-dalek = "4"  # Encryption
# Use same SQLite as grammers:
sqlite = "0.37"  # Match grammers-session
```

**Pros**:
- ✅ Single binary
- ✅ Both features enabled simultaneously
- ✅ No runtime overhead

**Cons**:
- ⚠️ Old library may not work with current WhatsApp protocol
- ⚠️ May need to build custom WhatsApp client
- ⚠️ Still unofficial (ToS risk)

**Effort**: 2-8 hours (test libraries) to 2-3 weeks (build custom)

---

### ⭐⭐ Option 2: Make SQLite Optional in WhatsApp Library

**Fork `whatsapp-rust` and make SQLite optional.**

#### Implementation:

1. **Fork whatsapp-rust**:
   ```bash
   git clone https://github.com/jlucaso1/whatsapp-rust
   cd whatsapp-rust
   ```

2. **Make SQLite optional**:
   ```toml
   # In whatsapp-rust/Cargo.toml
   [dependencies]
   rusqlite = { version = "0.35", optional = true }

   [features]
   default = ["sqlite-storage"]
   sqlite-storage = ["rusqlite"]
   ```

3. **Implement in-memory storage fallback**:
   ```rust
   #[cfg(feature = "sqlite-storage")]
   use rusqlite::Connection;

   pub enum SessionStore {
       #[cfg(feature = "sqlite-storage")]
       Sqlite(Connection),
       Memory(HashMap<String, Vec<u8>>),
   }
   ```

4. **Use your fork**:
   ```toml
   # In chat/Cargo.toml
   whatsapp-rust = { git = "https://github.com/YOUR_USERNAME/whatsapp-rust", branch = "no-sqlite", optional = true }
   ```

**Pros**:
- ✅ Single binary
- ✅ Both features work
- ✅ You control the fork

**Cons**:
- ⚠️ Need to maintain fork
- ⚠️ Memory-only storage = re-auth every restart
- ⚠️ Or implement SQLite storage using grammers' binding

**Effort**: 4-8 hours to fork and modify

---

### ⭐⭐⭐ Option 3: Use Grammers' SQLite for WhatsApp Too

**Make WhatsApp use the same `sqlite` crate as Telegram.**

#### Implementation:

1. **Fork whatsapp-rust** (or build custom WhatsApp client)

2. **Replace `rusqlite` with `sqlite`**:
   ```toml
   # Instead of:
   # rusqlite = "0.35"

   # Use:
   sqlite = "0.37"  # Same version as grammers-session
   ```

3. **Adapt session storage code**:
   ```rust
   // Old (rusqlite):
   let conn = Connection::open(path)?;
   conn.execute("CREATE TABLE ...", [])?;

   // New (sqlite):
   let db = sqlite::open(path)?;
   db.execute("CREATE TABLE ...")?;
   ```

**Pros**:
- ✅ Single binary
- ✅ Both features enabled
- ✅ Persistent session storage for both
- ✅ No runtime overhead

**Cons**:
- ⚠️ Need to fork and modify whatsapp-rust
- ⚠️ Must maintain compatibility with WhatsApp protocol
- ⚠️ Effort to port SQLite code

**Effort**: 1-2 days (if familiar with both libraries)

**This is the BEST long-term solution.**

---

### Option 4: Dynamic Library Approach

**Build WhatsApp as a separate dynamic library, load at runtime.**

#### Implementation:

1. **Create `libwhatsapp.so` shared library**:
   ```toml
   # crates/whatsapp-ffi/Cargo.toml
   [lib]
   crate-type = ["cdylib"]

   [dependencies]
   whatsapp-rust = "0.1.0-alpha"
   ```

2. **Define C FFI**:
   ```rust
   #[no_mangle]
   pub extern "C" fn wa_init(session_path: *const c_char) -> *mut WhatsAppClient {
       // ...
   }

   #[no_mangle]
   pub extern "C" fn wa_get_messages(client: *mut WhatsAppClient, ...) -> *mut MessageList {
       // ...
   }
   ```

3. **Load dynamically in main binary**:
   ```rust
   use libloading::{Library, Symbol};

   let lib = Library::new("libwhatsapp.so")?;
   let wa_init: Symbol<unsafe extern "C" fn(...) -> ...> = lib.get(b"wa_init")?;
   ```

**Pros**:
- ✅ Single binary (loads .so at runtime)
- ✅ Both platforms available
- ✅ No SQLite conflict

**Cons**:
- ❌ Complex FFI layer
- ❌ Cross-platform challenges (Windows .dll, macOS .dylib)
- ❌ Harder to debug
- ❌ Distribution complexity

**Effort**: 1-2 weeks

---

### Option 5: Process-Based Architecture

**Run WhatsApp as a separate process, communicate via IPC.**

#### Implementation:

1. **Create two binaries**:
   ```
   chat           # Main CLI (Telegram support)
   chat-whatsapp  # WhatsApp backend process
   ```

2. **IPC Communication** (choose one):
   - Unix sockets
   - Named pipes
   - HTTP (localhost)
   - gRPC

3. **Main binary spawns subprocess**:
   ```rust
   // In chat binary:
   use std::process::Command;

   fn get_whatsapp_messages(group: &str) -> Result<Vec<Message>> {
       // Start WhatsApp process if not running
       let output = Command::new("chat-whatsapp")
           .arg("get-messages")
           .arg(group)
           .output()?;

       // Parse JSON response
       serde_json::from_slice(&output.stdout)
   }
   ```

**Pros**:
- ✅ No dependency conflicts
- ✅ Process isolation
- ✅ Can restart WhatsApp without affecting Telegram
- ✅ Clear separation of concerns

**Cons**:
- ⚠️ Two binaries to distribute (but transparent to user)
- ⚠️ IPC overhead
- ⚠️ Process management complexity

**Effort**: 2-3 days

---

### Option 6: Conditional Compilation with Runtime Feature Detection

**Build both versions, load correct one at runtime.**

#### Implementation:

```rust
#[cfg(all(feature = "telegram", not(feature = "whatsapp")))]
pub mod platform {
    pub use crate::telegram_source::*;
}

#[cfg(all(feature = "whatsapp", not(feature = "telegram")))]
pub mod platform {
    pub use crate::whatsapp_source::*;
}

#[cfg(all(feature = "telegram", feature = "whatsapp"))]
compile_error!("Cannot enable both telegram and whatsapp features due to SQLite conflict. Build two binaries instead.");
```

Then build TWO binaries:
```bash
cargo build --features telegram -o chat-telegram
cargo build --features whatsapp -o chat-whatsapp
```

Create wrapper script:
```bash
#!/bin/bash
# chat (wrapper)
if [[ $2 == telegram:* ]]; then
    exec chat-telegram "$@"
elif [[ $2 == whatsapp:* ]]; then
    exec chat-whatsapp "$@"
else
    echo "Error: Specify platform (telegram: or whatsapp:)"
    exit 1
fi
```

**Pros**:
- ✅ Appears as single command to user
- ✅ No dependency conflicts
- ✅ Simple implementation

**Cons**:
- ❌ Two binaries to maintain
- ❌ 2x disk space
- ❌ Not truly a single program

**Effort**: 2-3 hours

---

## Comparison Matrix

| Option | Single Binary | No Conflicts | Effort | Maintenance | Recommended For |
|--------|---------------|--------------|--------|-------------|-----------------|
| 1. Alt Library | ✅ | ✅ | Low-High | Medium | Quick solution |
| 2. Optional SQLite | ✅ | ✅ | Medium | High | Temporary fix |
| 3. Unified SQLite | ✅ | ✅ | Medium | **Low** | **Production** ⭐ |
| 4. Dynamic Lib | ✅ | ✅ | High | High | Advanced users |
| 5. Multi-Process | ⚠️ | ✅ | Medium | Medium | Clean separation |
| 6. Wrapper Script | ❌ | ✅ | Low | Low | Quick MVP |

---

## My Recommendation

### For Your Use Case (Single WhatsApp Group):

**Start with Option 6 (Wrapper Script)** - Get working in 2-3 hours:
1. Build `--features whatsapp` as `chat-whatsapp`
2. Build `--features telegram` as `chat-telegram`
3. Create wrapper that dispatches based on platform prefix
4. User runs: `chat messages whatsapp:"Group" --limit 1000`

### For Production / Long-term:

**Option 3 (Unified SQLite)** - Best solution:
1. Fork `whatsapp-rust`
2. Replace `rusqlite` with `sqlite` crate (same as grammers)
3. Port session storage code (~1 day of work)
4. Submit PR upstream or maintain fork
5. Single binary, both platforms, persistent sessions

### Quick Test:

**Option 1 (Alt Library)** - Try old whatsappweb-rs:
```toml
whatsappweb-eta = { version = "0.1", optional = true }
```
If it compiles with telegram, you're done! If not, it needs updating.

---

## Action Plan for Single Binary

### Phase 1: Quick MVP (2-3 hours)
```bash
# Option 6: Build wrapper
cargo build --no-default-features --features whatsapp --bin chat
mv target/debug/chat target/debug/chat-whatsapp

cargo build --features telegram --bin chat
mv target/debug/chat target/debug/chat-telegram

# Create wrapper (see Option 6 above)
```

### Phase 2: Production Solution (1-2 days)
```bash
# Option 3: Unified SQLite
git clone https://github.com/jlucaso1/whatsapp-rust whatsapp-rust-fork
cd whatsapp-rust-fork
# Replace rusqlite with sqlite crate
# Port session storage code
# Test
# Use fork in Cargo.toml
```

---

## Next Steps

**Choose your path**:

1. **Need it working TODAY?** → Option 6 (wrapper script)
2. **Want proper solution?** → Option 3 (unified SQLite fork)
3. **Want to test quick?** → Option 1 (try whatsappweb-eta)
4. **Want cleanest arch?** → Option 5 (multi-process)

**Which option would you like me to implement?**
