# Technical Design: PTY Terminal Support

## Context

The terminal tab currently uses `std::process::Command` with piped stdio, which works for simple commands but breaks for interactive programs. Programs like `claude`, Python/Node REPLs, and interactive shells detect the absence of a TTY and change behavior (disable prompts, buffer output, or exit).

Current architecture (src/ui/app.rs:113-214):
- Spawns process via `sh -c <command>`
- Captures stdout/stderr via `Stdio::piped()`
- Reads output in separate threads using byte buffers
- Sends input via channel to stdin writer thread

**Constraints:**
- Must maintain existing debug output and error handling
- Must work across platforms (Linux, macOS, Windows)
- Must integrate with existing egui UI rendering loop
- Cannot block UI thread

## Goals / Non-Goals

### Goals
- Enable interactive CLI programs (claude, python, node, ssh, etc.)
- Provide proper TTY emulation with ISATTY detection
- Support terminal colors and control sequences
- Handle terminal resize events
- Maintain existing debug logging for troubleshooting

### Non-Goals
- Full VT100/ANSI terminal emulation (use basic PTY)
- Terminal multiplexing (tmux/screen-like features)
- Copy/paste or selection (future enhancement)
- Scrollback buffer management (rely on egui ScrollArea)

## Decisions

### Decision 1: Use portable-pty crate

**Rationale:**
- Cross-platform PTY abstraction (works on Linux, macOS, Windows via ConPTY)
- Well-maintained, used in production (Wezterm terminal emulator)
- Simple API: `PtySystem`, `PtyPair`, `MasterPty`, `SlavePty`
- Handles platform-specific PTY setup (posix_openpt, CreatePseudoConsole)

**Alternatives considered:**
- `pty` crate - Unix-only, not actively maintained
- `conpty` + manual Unix PTY - more code, platform-specific bugs
- Stay with pipes + detection hacks - doesn't solve core TTY issue

### Decision 2: Terminal size handling

Set initial size to 80x24 (standard default), update on window resize:
```rust
let pty_size = PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
};
```

Resize on UI changes (calculate from output area dimensions):
```rust
// In render_term_tab, calculate rows/cols from available space
let rows = (output_height / line_height) as u16;
let cols = (output_width / char_width) as u16;
master.resize(PtySize { rows, cols, .. })?;
```

### Decision 3: I/O architecture

Keep thread-based I/O model but adapt for PTY:
- **Main thread:** UI rendering, channel polling
- **Reader thread:** Read from master PTY, send to output channel
- **Writer thread:** Receive from input channel, write to master PTY

```rust
let pair = pty_system.openpty(pty_size)?;
let mut master = pair.master;
let slave = pair.slave;

// Spawn child with slave PTY as stdio
let mut child = slave.spawn_command(cmd)?;

// Reader thread
std::thread::spawn(move || {
    let mut reader = master.try_clone_reader()?;
    let mut buf = [0u8; 1024];
    loop {
        match reader.read(&mut buf) {
            Ok(n) if n > 0 => output_tx.send(...)?,
            _ => break,
        }
    }
});
```

### Decision 4: Debug output preservation

Maintain `[DEBUG]` messages by wrapping PTY operations:
- Process start/exit status
- I/O thread lifecycle
- Read/write byte counts
- Errors and warnings

This preserves troubleshooting capability while fixing interactive programs.

## Risks / Trade-offs

### Risk: Platform-specific PTY behavior
- **Mitigation:** Use portable-pty which abstracts platform differences
- **Fallback:** Log platform-specific errors clearly, consider graceful degradation

### Risk: Terminal size mismatch
- Programs may assume 80x24 but UI area might be different
- **Mitigation:** Calculate size from available UI space, send SIGWINCH on resize

### Risk: Performance with high-throughput programs
- Programs outputting MB/s of data could overwhelm UI
- **Mitigation:** Use buffered channels (bounded size), consider rate limiting in future

### Trade-off: Complexity vs functionality
- PTY adds dependency and complexity
- **Justification:** Essential for interactive programs, no simpler solution exists

## Migration Plan

### Phase 1: Add PTY support (this change)
1. Add `portable-pty` dependency
2. Create PTY-based spawning function
3. Update I/O threads for PTY reader/writer
4. Test with claude, python, bash

### Phase 2: Refinement (future)
- Dynamic terminal size based on UI dimensions
- Better error messages for PTY failures
- Optional: Raw mode toggle for binary data

### Rollback Plan
If PTY causes critical issues:
1. Add feature flag: `--features pty-terminal`
2. Conditionally compile PTY code
3. Fall back to pipe-based implementation
4. Document known limitations

## Open Questions

1. **Q:** Should we expose terminal size in UI for user adjustment?
   **A:** Not initially - calculate from available space, add manual control later if needed

2. **Q:** How to handle programs that change terminal mode (raw mode, no echo)?
   **A:** PTY handles this automatically, no special handling needed

3. **Q:** What about Windows CMD.exe vs PowerShell?
   **A:** portable-pty supports both via ConPTY, test on Windows during implementation

## References

- `portable-pty` documentation: https://docs.rs/portable-pty
- PTY overview: https://en.wikipedia.org/wiki/Pseudoterminal
- Current implementation: src/ui/app.rs:113-214
