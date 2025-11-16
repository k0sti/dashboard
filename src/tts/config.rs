/// TTS configuration and data types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Voice identifier (model name)
pub type VoiceId = String;

/// Voice quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceQuality {
    Low,
    Medium,
    High,
}

/// Voice model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMetadata {
    /// Unique identifier
    pub id: VoiceId,
    /// Display name
    pub name: String,
    /// Language code (e.g., "en-US")
    pub language: String,
    /// Quality level
    pub quality: VoiceQuality,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Path to ONNX model file
    pub onnx_path: PathBuf,
    /// Path to config JSON file
    pub config_path: PathBuf,
}

/// TTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSConfig {
    /// Whether TTS is enabled
    pub enabled: bool,
    /// Auto-speak mode (automatically speak new agent messages)
    pub auto_speak: bool,
    /// Selected voice model ID
    pub selected_voice: VoiceId,
    /// Playback speed (0.5 to 2.0)
    pub playback_speed: f32,
    /// Audio device name (None = default device)
    pub audio_device: Option<String>,
    /// Directory containing Piper voice models
    pub model_directory: PathBuf,
}

impl Default for TTSConfig {
    fn default() -> Self {
        // Use XDG config directory or fallback to ~/.config
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("agent-dashboard")
            .join("tts")
            .join("models");

        Self {
            enabled: false,
            auto_speak: false,
            selected_voice: "default".to_string(),
            playback_speed: 1.0,
            audio_device: None,
            model_directory: config_dir,
        }
    }
}

impl TTSConfig {
    /// Validate configuration values
    pub fn validate(&mut self) {
        self.playback_speed = self.playback_speed.clamp(0.5, 2.0);
    }
}
