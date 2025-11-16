/// Audio playback with WAV file generation
///
/// This implementation saves generated audio to WAV files for playback.
/// Files are saved to ~/.config/agent-dashboard/tts/audio/
///
/// Future enhancement: Use rodio (optional feature) for direct playback

use anyhow::{Result, Context};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use hound;

/// Audio playback manager (stub implementation)
#[derive(Clone)]
pub struct AudioPlayer {
    playing: Arc<AtomicBool>,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Result<Self> {
        log::info!("TTS Audio Player initialized (WAV file output mode)");

        // Create audio output directory
        let audio_dir = Self::get_audio_dir()?;
        if !audio_dir.exists() {
            fs::create_dir_all(&audio_dir)
                .context("Failed to create audio output directory")?;
            log::info!("Created audio directory: {:?}", audio_dir);
        }

        Ok(Self {
            playing: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the audio output directory
    fn get_audio_dir() -> Result<PathBuf> {
        use crate::config::AppConfig;
        let config_dir = AppConfig::config_dir()?;
        Ok(config_dir.join("tts").join("audio"))
    }

    /// Play audio samples by saving to WAV file
    pub fn play(&self, samples: Vec<f32>, sample_rate: u32, speed: f32) -> Result<()> {
        self.playing.store(true, Ordering::SeqCst);

        // Apply speed adjustment by resampling
        let adjusted_samples = if (speed - 1.0).abs() > 0.01 {
            Self::adjust_speed(&samples, speed)
        } else {
            samples
        };

        let duration_secs = adjusted_samples.len() as f32 / sample_rate as f32;
        log::info!(
            "Generating TTS audio: {} samples at {}Hz (speed: {}x, duration: {:.2}s)",
            adjusted_samples.len(),
            sample_rate,
            speed,
            duration_secs
        );

        // Save to WAV file
        let audio_path = Self::get_audio_dir()?.join(format!(
            "tts_{}.wav",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        Self::save_wav(&audio_path, &adjusted_samples, sample_rate)?;
        log::info!("Saved TTS audio to: {:?}", audio_path);

        Ok(())
    }

    /// Save audio samples to WAV file
    fn save_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)
            .context("Failed to create WAV file")?;

        // Convert f32 samples (-1.0 to 1.0) to i16 samples
        for &sample in samples {
            let amplitude = (sample.max(-1.0).min(1.0) * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)
                .context("Failed to write WAV sample")?;
        }

        writer.finalize()
            .context("Failed to finalize WAV file")?;

        Ok(())
    }

    /// Adjust playback speed by simple resampling
    fn adjust_speed(samples: &[f32], speed: f32) -> Vec<f32> {
        if (speed - 1.0).abs() < 0.01 {
            return samples.to_vec();
        }

        let new_len = (samples.len() as f32 / speed) as usize;
        let mut result = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = (i as f32 * speed) as usize;
            if src_idx < samples.len() {
                result.push(samples[src_idx]);
            }
        }

        result
    }

    /// Stop playback
    pub fn stop(&self) {
        log::debug!("TTS playback stopped");
        self.playing.store(false, Ordering::SeqCst);
    }

    /// Pause playback
    #[allow(dead_code)]
    pub fn pause(&self) {
        log::debug!("TTS playback paused");
    }

    /// Resume playback
    #[allow(dead_code)]
    pub fn resume(&self) {
        log::debug!("TTS playback resumed");
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::SeqCst)
    }

    /// Wait for playback to complete (simulated)
    pub fn wait_for_completion(&self) {
        if self.is_playing() {
            // Simulate a brief playback duration
            std::thread::sleep(Duration::from_millis(500));
            self.playing.store(false, Ordering::SeqCst);
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create audio player")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_creation() {
        let player = AudioPlayer::new().unwrap();
        assert!(!player.is_playing());
    }

    #[test]
    fn test_play_and_stop() {
        let player = AudioPlayer::new().unwrap();
        let samples = vec![0.0; 1000];
        player.play(samples, 22050, 1.0).unwrap();
        assert!(player.is_playing());
        player.stop();
        assert!(!player.is_playing());
    }
}
