# Change: Add Text-to-Speech to Chat Interface

## Why

Users interacting with multiple AI agents need hands-free operation and auditory feedback, especially during long conversations or when multitasking. Reading long agent responses can be fatiguing and reduces productivity. Text-to-speech capabilities enable users to listen to agent responses while working on other tasks, improving accessibility and user experience.

## What Changes

- Add TTS engine capability using Candle (Rust ML framework) and Piper TTS (neural TTS system)
- Add audio playback controls to chat interface (play, pause, stop)
- Add per-message TTS controls (speak individual messages)
- Add auto-speak mode for new agent messages
- Add voice selection and playback speed configuration
- Modify chat interface to integrate TTS controls and audio state indicators
- Add audio queue management for sequential message playback

## Impact

- Affected specs:
  - `chat-interface` (modified) - add TTS controls and audio state
  - `tts-engine` (new) - core TTS functionality using Candle and Piper

- Affected code:
  - Chat interface UI components (add TTS buttons and controls)
  - Audio playback subsystem (Candle model loading, Piper TTS inference)
  - Message processing pipeline (optional auto-speak integration)
  - Configuration system (voice models, playback settings)
  - New dependencies: `candle-core`, `candle-nn`, `rodio` or `cpal` for audio output
