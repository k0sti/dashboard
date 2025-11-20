# Change: Add PTY Support for Interactive Terminal Programs

## Why

The current terminal implementation uses standard pipes (`stdin`/`stdout`/`stderr`) which causes interactive CLI programs (like `claude`, `python`, `node`, etc.) to fail or produce no output. These programs detect they are not running in a real terminal (TTY) and either:
- Disable output completely
- Exit immediately
- Change behavior to non-interactive mode
- Buffer output indefinitely

Users cannot run interactive commands in the Term tab, severely limiting its usefulness for command-line workflows.

## What Changes

- Replace pipe-based process spawning with pseudo-terminal (PTY) allocation using `portable-pty` crate
- Implement PTY reader/writer for terminal I/O with proper terminal emulation
- Add terminal size negotiation and resize handling
- Maintain backward compatibility with debug output and existing UI
- Support for interactive programs that require TTY detection (ISATTY checks)

## Impact

### Affected Specs
- **NEW**: `terminal-interface` - Terminal emulation and process management capability

### Affected Code
- `src/ui/app.rs` - Terminal spawning and I/O handling in `spawn_terminal()` and `render_term_tab()`
- `Cargo.toml` - Add `portable-pty` dependency

### Benefits
- Interactive CLI tools (claude, python REPL, node REPL) will work correctly
- Better compatibility with shell features (job control, terminal colors)
- More accurate terminal emulation for complex programs

### Risks
- PTY behavior differs slightly across platforms (Linux/macOS/Windows)
- Need to handle terminal size changes and window resizing
- More complex error handling for PTY operations
