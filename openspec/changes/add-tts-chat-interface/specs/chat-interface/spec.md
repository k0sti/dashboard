# Chat Interface TTS Modifications

## ADDED Requirements

### Requirement: TTS Message Controls
The system SHALL provide text-to-speech controls for each message in the chat log.

#### Scenario: Speak individual message
- **WHEN** user clicks the speak button on a message
- **THEN** the TTS engine SHALL convert the message text to speech
- **AND** audio SHALL play through the system default audio device
- **AND** the message SHALL display a visual indicator showing it is being spoken

#### Scenario: Stop speaking message
- **WHEN** a message is currently being spoken
- **AND** user clicks the stop button
- **THEN** audio playback SHALL immediately stop
- **AND** the speaking indicator SHALL be removed

#### Scenario: Multiple speak requests
- **WHEN** user clicks speak on a message while another message is playing
- **THEN** the current message SHALL stop playing
- **AND** the new message SHALL begin playing immediately

### Requirement: Auto-Speak Mode
The system SHALL support automatic text-to-speech for new agent messages.

#### Scenario: Enable auto-speak for agent responses
- **WHEN** user enables auto-speak mode
- **AND** an agent sends a new message
- **THEN** the message SHALL automatically be converted to speech and played
- **AND** the message SHALL be added to the audio queue if another message is playing

#### Scenario: Disable auto-speak mode
- **WHEN** user disables auto-speak mode
- **THEN** new agent messages SHALL NOT automatically trigger TTS
- **AND** currently playing audio SHALL continue to completion
- **AND** queued messages SHALL be cleared

#### Scenario: Auto-speak queue management
- **WHEN** auto-speak is enabled
- **AND** multiple agents send messages in quick succession
- **THEN** messages SHALL be queued in chronological order
- **AND** each message SHALL play sequentially to completion
- **AND** user SHALL be able to view and manage the audio queue

### Requirement: TTS Configuration Controls
The system SHALL provide configuration options for text-to-speech playback.

#### Scenario: Voice model selection
- **WHEN** user opens TTS settings
- **THEN** system SHALL display available Piper voice models
- **AND** user SHALL be able to select a preferred voice
- **AND** selected voice SHALL persist across sessions
- **AND** voice change SHALL apply to subsequent TTS requests

#### Scenario: Playback speed control
- **WHEN** user adjusts playback speed setting
- **THEN** system SHALL support speed range from 0.5x to 2.0x
- **AND** speed change SHALL apply immediately to active playback
- **AND** speed setting SHALL persist across sessions

#### Scenario: Audio output device selection
- **WHEN** user opens TTS settings
- **THEN** system SHALL list available audio output devices
- **AND** user SHALL be able to select output device
- **AND** TTS audio SHALL route to selected device

### Requirement: Audio State Indicators
The system SHALL provide visual feedback for TTS playback state.

#### Scenario: Display playback status
- **WHEN** TTS is playing a message
- **THEN** the message SHALL show an animated speaker icon
- **AND** chat interface SHALL show global playback status indicator
- **AND** status SHALL display current message being spoken

#### Scenario: Display audio queue
- **WHEN** messages are queued for playback
- **THEN** system SHALL display queue count
- **AND** user SHALL be able to view queued message list
- **AND** user SHALL be able to clear the queue
- **AND** user SHALL be able to skip to next message in queue
