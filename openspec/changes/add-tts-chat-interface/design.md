# TTS Chat Interface Design

## Context

This design covers the integration of text-to-speech capabilities into the Autonomous Agent Dashboard using Rust-native ML inference (Candle) and Piper TTS models. The system must support real-time audio synthesis, playback controls, and queue management while maintaining UI responsiveness.

**Constraints:**
- Must work with existing egui-based UI architecture
- Must maintain 60fps UI rendering during TTS operations
- Must support cross-platform audio output (Linux, macOS, Windows)
- Must work with locally-stored Piper ONNX models (no cloud dependencies)
- Memory footprint should remain reasonable (~500MB per loaded model)

**Stakeholders:**
- End users requiring hands-free operation and accessibility
- Developers maintaining the agent dashboard codebase

## Goals / Non-Goals

### Goals
- Provide low-latency text-to-speech (<600ms from request to audio start for short messages)
- Support multiple Piper voice models with hot-swapping
- Enable auto-speak mode for hands-free operation
- Maintain UI responsiveness during synthesis and playback
- Provide intuitive playback controls and queue management
- Support playback speed adjustment without pitch distortion

### Non-Goals
- Real-time voice cloning or custom voice training
- Cloud-based TTS APIs or streaming synthesis
- Video/avatar lip-sync integration
- Voice activity detection or speech recognition (future work)
- Multi-language translation (only using pre-trained Piper models)

## Decisions

### Decision 1: Use Candle for ML Inference
**Choice:** Use Candle (Rust-native ML framework) instead of Python-based solutions or FFI bindings to ONNX Runtime.

**Rationale:**
- Pure Rust solution aligns with project tech stack (no Python dependencies)
- Candle supports ONNX model loading and inference
- Native performance without FFI overhead
- Better integration with Tokio async runtime
- Smaller deployment footprint (no Python interpreter)

**Alternatives considered:**
- **ONNX Runtime C API with Rust bindings:** More mature but requires C library dependencies, complicates cross-platform builds
- **PyO3 + Python TTS libraries:** Easy integration with piper-tts Python implementation but adds Python runtime dependency
- **Mimic3 or other Rust TTS engines:** Limited compared to Piper's voice quality and model availability

### Decision 2: Use Rodio for Audio Playback
**Choice:** Use `rodio` library for cross-platform audio playback.

**Rationale:**
- High-level audio playback API (simpler than `cpal`)
- Built on top of `cpal` for cross-platform support
- Handles common audio formats and resampling
- Easy integration with Rust async ecosystem
- Active maintenance and good documentation

**Alternatives considered:**
- **cpal (low-level audio):** More control but requires managing audio streams, mixing, and device handling manually
- **SDL2 audio:** Adds large dependency for simple audio playback needs

### Decision 3: Message Queue Architecture
**Choice:** Implement in-memory FIFO queue with async processing using Tokio channels.

**Design:**
```rust
// Pseudo-code structure
struct TTSQueue {
    tx: mpsc::Sender<TTSRequest>,
    rx: mpsc::Receiver<TTSRequest>,
    status: Arc<RwLock<QueueStatus>>,
}

struct TTSRequest {
    message_id: String,
    text: String,
    priority: Priority,
}
```

**Rationale:**
- FIFO ensures chronological playback of agent messages
- Tokio channels provide thread-safe async communication
- Bounded channel (capacity 50) prevents unbounded memory growth
- Separate synthesis and playback tasks prevent blocking

**Alternatives considered:**
- **Priority queue:** Adds complexity without clear benefit for chat use case
- **Database-backed queue:** Overkill for ephemeral audio playback

### Decision 4: Threading Model
**Choice:** Use three async tasks:
1. **TTS Service Task:** Receives requests, manages queue, coordinates synthesis and playback
2. **Synthesis Task:** Runs Candle inference, produces audio samples
3. **Playback Task:** Consumes audio samples, streams to audio device

**Rationale:**
- Separates concerns (request handling, computation, I/O)
- Synthesis task can be CPU-intensive without blocking UI
- Playback task handles real-time audio streaming independently
- Tokio async tasks integrate cleanly with existing agent architecture

**Flow:**
```
UI Thread (egui)
  ↓ send TTS request
TTS Service Task
  ↓ queue request
Synthesis Task (spawn_blocking for Candle)
  ↓ audio samples (channel)
Playback Task (rodio sink)
  ↓ audio output
System Audio Device
```

### Decision 5: Model Loading Strategy
**Choice:** Lazy load models on first use, cache in memory, support hot-swap with reference counting.

**Design:**
```rust
struct ModelCache {
    current: Arc<RwLock<Option<PiperModel>>>,
    registry: HashMap<VoiceId, ModelMetadata>,
}
```

**Rationale:**
- Lazy loading reduces startup time
- Single model in memory reduces footprint (~500MB per model)
- Hot-swap allows voice changes without restart
- Reference counting prevents unloading model during active synthesis

**Alternatives considered:**
- **Preload all models:** Wastes memory if user only uses one voice
- **Load per request:** Adds significant latency (model loading takes 1-3 seconds)

### Decision 6: Text Preprocessing
**Choice:** Implement minimal preprocessing (punctuation normalization, sentence splitting) without full NLP pipeline.

**Rationale:**
- Piper models handle most prosody internally
- Sentence splitting enables chunking for long messages
- Minimal processing reduces latency
- Common abbreviation expansion improves quality without complexity

**Preprocessing steps:**
1. Normalize Unicode characters
2. Expand common abbreviations (Dr., Mr., etc.)
3. Split on sentence boundaries (`.`, `!`, `?`)
4. Filter unsupported characters (Piper typically supports basic Latin + language-specific)

### Decision 7: Playback Speed Implementation
**Choice:** Use time-stretching algorithm (e.g., `rubato` or `sonic` library) for pitch-preserving speed adjustment.

**Rationale:**
- Simple sample rate changes create "chipmunk effect"
- Time-stretching maintains pitch while changing duration
- Support 0.5x to 2.0x range covers most use cases
- Slightly increases latency but preserves quality

**Alternatives considered:**
- **Resample at synthesis time:** Would require re-running Candle inference, too slow
- **Simple speed change:** Acceptable quality loss for speech

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    egui UI Thread                       │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ Chat Panel  │  │ TTS Controls │  │ TTS Settings  │  │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘  │
└─────────┼────────────────┼──────────────────┼──────────┘
          │ Speak Request  │ Play/Stop/Skip   │ Config
          ▼                ▼                  ▼
┌─────────────────────────────────────────────────────────┐
│              TTS Service (Tokio Task)                   │
│  ┌──────────────┐    ┌─────────────┐   ┌────────────┐  │
│  │ Request      │───▶│ Audio Queue │──▶│ Playback   │  │
│  │ Handler      │    │ (FIFO)      │   │ Controller │  │
│  └──────────────┘    └─────────────┘   └────────────┘  │
│         │                                      │        │
│         ▼                                      ▼        │
│  ┌──────────────┐                      ┌────────────┐  │
│  │ Model Cache  │                      │ Audio Sink │  │
│  │ (Candle)     │                      │ (rodio)    │  │
│  └──────┬───────┘                      └────────────┘  │
│         │                                      │        │
└─────────┼──────────────────────────────────────┼───────┘
          │ spawn_blocking                       │
          ▼                                      ▼
┌──────────────────┐                    ┌─────────────────┐
│ Synthesis Task   │                    │ Audio Device    │
│ (Candle Inference)│                   │ (System Output) │
└──────────────────┘                    └─────────────────┘
```

## Data Models

### TTS Configuration
```rust
#[derive(Serialize, Deserialize, Clone)]
struct TTSConfig {
    enabled: bool,
    auto_speak: bool,
    selected_voice: VoiceId,
    playback_speed: f32,         // 0.5 to 2.0
    audio_device: Option<String>, // None = default device
    model_directory: PathBuf,
}
```

### Voice Model Metadata
```rust
struct VoiceMetadata {
    id: VoiceId,
    name: String,
    language: String,
    quality: VoiceQuality,  // Low, Medium, High
    sample_rate: u32,
    onnx_path: PathBuf,
    config_path: PathBuf,
}
```

### TTS Request
```rust
struct TTSRequest {
    message_id: Uuid,
    text: String,
    voice_id: VoiceId,
    speed: f32,
}
```

### Queue Status
```rust
struct QueueStatus {
    current: Option<TTSRequest>,
    queued: VecDeque<TTSRequest>,
    playing: bool,
}
```

## Risks / Trade-offs

### Risk 1: Model Loading Latency
**Risk:** First TTS request takes 1-3 seconds while loading Piper model.

**Mitigation:**
- Preload default model on app startup in background task
- Show loading indicator in UI during model swap
- Cache loaded model in memory for subsequent requests

### Risk 2: Memory Usage with Large Models
**Risk:** High-quality Piper models can consume 300-500MB RAM per model.

**Mitigation:**
- Load only one model at a time
- Provide model quality selection (low/medium/high)
- Document memory requirements in user guide
- Consider memory-mapped model loading if Candle supports it

### Risk 3: Audio Device Compatibility
**Risk:** Audio output may fail on some systems due to device configuration or driver issues.

**Mitigation:**
- Use rodio's default device selection as fallback
- Provide device selection UI with error handling
- Gracefully degrade to "TTS unavailable" state
- Log detailed audio device errors for troubleshooting

### Risk 4: Real-time Performance on Low-end Hardware
**Risk:** Synthesis may be slow on low-end CPUs, causing noticeable latency.

**Mitigation:**
- Target <500ms synthesis for typical messages (100 chars)
- Use Candle's CPU optimizations (SIMD, threading)
- Consider GPU acceleration path for supported systems (future)
- Provide performance benchmarking utility

### Risk 5: Piper Model Availability
**Risk:** Users may not have Piper models installed, causing TTS to be non-functional out-of-box.

**Mitigation:**
- Bundle one small/fast Piper model with distribution (~50MB)
- Provide clear installation instructions for additional voices
- Detect missing models on startup and show helpful error message
- Consider automated download utility (future enhancement)

### Trade-off: Candle vs Python-based TTS
**Trade-off:** Using Candle limits us to ONNX models and requires Rust ML expertise, whereas Python-based solutions offer more mature ecosystems.

**Justification:**
- Aligns with project's Rust-first philosophy
- Eliminates Python runtime dependency
- Better long-term maintainability with single language stack
- Candle is actively developed by Hugging Face team

### Trade-off: In-Memory Queue vs Persistent Queue
**Trade-off:** Audio queue is ephemeral and cleared on app restart.

**Justification:**
- Audio playback is real-time and ephemeral by nature
- Persisting audio queue adds complexity without clear user benefit
- Users can regenerate TTS from chat history if needed

## Migration Plan

N/A - This is a new feature with no existing TTS implementation to migrate from.

**Installation steps for users:**
1. Update to version with TTS support
2. Download desired Piper voice models from official repository
3. Place models in `~/.config/agent-dashboard/tts/models/` directory
4. Configure voice selection in TTS settings panel

**Rollback:**
- TTS is an optional feature; disabling it in settings reverts to text-only chat
- No data migration or schema changes required

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| First TTS latency (cold start) | < 3s | Time from app start to first audio |
| TTS latency (warm, short message <100 chars) | < 600ms | Request to audio start |
| TTS latency (long message ~1000 chars) | < 5s | Total synthesis time |
| Model memory usage | < 500MB | Per loaded model |
| UI framerate during TTS | 60fps | egui render time |
| Audio queue capacity | 50 messages | Maximum queued items |

## Open Questions

1. **Should we support SSML (Speech Synthesis Markup Language) for advanced prosody control?**
   - Defer to future enhancement if user demand exists
   - Piper models have limited SSML support

2. **Should we support multiple concurrent audio streams (overlapping agent voices)?**
   - No for V1 - adds complexity without clear benefit
   - Sequential playback is more comprehensible

3. **Should we provide voice model download/management UI?**
   - Defer to V2 - manual installation is acceptable for V1
   - Focus on core TTS functionality first

4. **Should we cache synthesized audio for repeated messages?**
   - Yes, but with memory limits (e.g., cache last 20 messages)
   - Implement in V1.1 if performance data shows benefit

5. **Should we support audio output to file (export TTS as WAV/MP3)?**
   - Defer to future enhancement
   - Not a core requirement for hands-free chat interaction
