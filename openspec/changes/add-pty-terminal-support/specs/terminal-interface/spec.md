# Terminal Interface Capability

## ADDED Requirements

### Requirement: PTY-based Process Spawning
The system SHALL spawn terminal processes using pseudo-terminal (PTY) allocation to enable proper TTY emulation for interactive programs.

#### Scenario: Interactive program detects TTY
- **GIVEN** user sets startup command to "python" (interactive REPL)
- **WHEN** process spawns
- **THEN** python detects a TTY and displays interactive prompt
- **AND** ISATTY checks return true

#### Scenario: Non-interactive command works
- **GIVEN** user sets startup command to "echo hello"
- **WHEN** process spawns
- **THEN** command executes successfully via PTY
- **AND** output "hello" appears in terminal

### Requirement: Terminal Size Configuration
The system SHALL configure PTY with appropriate terminal dimensions and handle resize events.

#### Scenario: Default terminal size
- **GIVEN** no explicit size configuration
- **WHEN** PTY is created
- **THEN** terminal size is set to 80 columns x 24 rows (standard default)

#### Scenario: Terminal resize on UI change
- **GIVEN** terminal output area dimensions change
- **WHEN** UI recalculates available space
- **THEN** PTY size is updated to match new dimensions
- **AND** child process receives SIGWINCH signal (Unix) or resize notification (Windows)

### Requirement: PTY I/O Handling
The system SHALL read from and write to PTY master in separate threads without blocking the UI.

#### Scenario: Reading PTY output
- **GIVEN** child process writes to stdout/stderr
- **WHEN** PTY master has data available
- **THEN** reader thread reads bytes immediately
- **AND** output is sent to UI via channel
- **AND** UI renders output without delay

#### Scenario: Writing to PTY input
- **GIVEN** user types command and presses Enter
- **WHEN** input is sent to PTY writer
- **THEN** bytes are written to PTY master
- **AND** child process receives input on stdin

#### Scenario: PTY EOF handling
- **GIVEN** child process exits
- **WHEN** PTY master returns EOF on read
- **THEN** reader thread terminates gracefully
- **AND** exit status is displayed to user

### Requirement: Cross-Platform PTY Support
The system SHALL use portable-pty crate to abstract platform-specific PTY implementations.

#### Scenario: Linux PTY allocation
- **GIVEN** running on Linux
- **WHEN** spawning terminal process
- **THEN** PTY is allocated via posix_openpt
- **AND** interactive programs work correctly

#### Scenario: macOS PTY allocation
- **GIVEN** running on macOS
- **WHEN** spawning terminal process
- **THEN** PTY is allocated via macOS PTY APIs
- **AND** interactive programs work correctly

#### Scenario: Windows ConPTY allocation
- **GIVEN** running on Windows 10+
- **WHEN** spawning terminal process
- **THEN** PTY is allocated via ConPTY (Windows Console API)
- **AND** interactive programs work correctly

### Requirement: Debug Logging for PTY Operations
The system SHALL provide debug output for PTY lifecycle and I/O operations to aid troubleshooting.

#### Scenario: PTY creation debug output
- **GIVEN** debug mode is enabled
- **WHEN** PTY is allocated
- **THEN** startup message shows command being executed
- **AND** process PID is displayed
- **AND** PTY size (rows x cols) is logged

#### Scenario: I/O thread lifecycle logging
- **GIVEN** debug mode is enabled
- **WHEN** I/O threads start/stop
- **THEN** thread lifecycle events are logged
- **AND** byte counts for read/write operations are shown
- **AND** errors include descriptive messages

### Requirement: Terminal Reset on Startup Command Change
The system SHALL terminate existing PTY and spawn new process when startup command is modified.

#### Scenario: Change from bash to python
- **GIVEN** terminal is running with "bash" startup command
- **WHEN** user changes startup command to "python" and presses Enter
- **THEN** existing bash process is terminated
- **AND** PTY is closed and recreated
- **AND** new python REPL starts with fresh PTY
- **AND** output area is cleared

#### Scenario: Process termination cleanup
- **GIVEN** terminal process is running
- **WHEN** startup command is changed
- **THEN** SIGTERM is sent to child process (Unix) or TerminateProcess (Windows)
- **AND** PTY master is closed
- **AND** I/O threads terminate gracefully

### Requirement: Error Handling for PTY Failures
The system SHALL display clear error messages when PTY operations fail.

#### Scenario: PTY allocation failure
- **GIVEN** PTY allocation fails (e.g., resource exhaustion)
- **WHEN** attempting to spawn process
- **THEN** error message is displayed in red
- **AND** message includes reason for failure
- **AND** terminal remains usable for retry

#### Scenario: Process spawn failure
- **GIVEN** PTY is allocated successfully
- **WHEN** command spawn fails (e.g., command not found)
- **THEN** error message shows command that failed
- **AND** error includes system error description
- **AND** PTY is cleaned up properly

#### Scenario: I/O read error
- **GIVEN** PTY master read operation fails
- **WHEN** reader thread encounters I/O error
- **THEN** error is logged with details
- **AND** thread terminates gracefully
- **AND** exit status is displayed
