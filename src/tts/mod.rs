/// Text-to-Speech module for Ollama chat interface
///
/// This module provides TTS capabilities using Candle (Rust ML framework)
/// and Piper TTS models for converting agent messages to speech.

pub mod config;
pub mod model;
pub mod synthesis;
pub mod playback;
pub mod queue;
pub mod service;

pub use config::{TTSConfig, VoiceId};
pub use service::TTSService;

use uuid::Uuid;

/// TTS request for synthesizing and playing text
#[derive(Debug, Clone)]
pub struct TTSRequest {
    /// Unique identifier for this request
    #[allow(dead_code)]
    pub message_id: Uuid,
    /// Text to synthesize
    pub text: String,
    /// Voice model to use
    pub voice_id: VoiceId,
    /// Playback speed (0.5 to 2.0)
    pub speed: f32,
}

impl TTSRequest {
    pub fn new(text: String, voice_id: VoiceId, speed: f32) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            text,
            voice_id,
            speed: speed.clamp(0.5, 2.0),
        }
    }
}

/// Current status of the TTS queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QueueStatus {
    /// Currently playing request
    pub current: Option<TTSRequest>,
    /// Number of queued requests
    pub queue_length: usize,
    /// Whether audio is currently playing
    pub playing: bool,
}

impl Default for QueueStatus {
    fn default() -> Self {
        Self {
            current: None,
            queue_length: 0,
            playing: false,
        }
    }
}

/// Commands that can be sent to the TTS service
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TTSCommand {
    /// Speak the given text
    Speak(TTSRequest),
    /// Stop current playback
    Stop,
    /// Skip to next in queue
    Skip,
    /// Clear the queue
    ClearQueue,
    /// Get current status
    GetStatus,
    /// Shutdown the service
    Shutdown,
}

/// TTS service responses
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TTSResponse {
    /// Status response
    Status(QueueStatus),
    /// Operation completed successfully
    Ok,
    /// Error occurred
    Error(String),
}
