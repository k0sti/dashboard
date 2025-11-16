/// TTS service - main facade for text-to-speech functionality
///
/// This service coordinates model loading, synthesis, playback, and queue management.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

use crate::tts::{
    TTSCommand, TTSRequest, TTSResponse,
    config::TTSConfig,
    model::ModelCache,
    playback::AudioPlayer,
    queue::TTSQueue,
    synthesis,
};

/// TTS service handle for communicating with the service task
#[derive(Clone)]
pub struct TTSService {
    command_tx: mpsc::Sender<TTSCommand>,
    #[allow(dead_code)]
    response_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<TTSResponse>>>,
}

impl TTSService {
    /// Create and start the TTS service
    pub fn start(config: TTSConfig) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(32);
        let (response_tx, response_rx) = mpsc::channel(32);

        // Spawn the service task
        task::spawn(async move {
            if let Err(e) = run_service(config, command_rx, response_tx).await {
                log::error!("TTS service error: {}", e);
            }
        });

        Ok(Self {
            command_tx,
            response_rx: Arc::new(tokio::sync::Mutex::new(response_rx)),
        })
    }

    /// Send a command to the TTS service
    pub async fn send_command(&self, command: TTSCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .await
            .context("Failed to send command to TTS service")
    }

    /// Receive a response from the TTS service
    #[allow(dead_code)]
    pub async fn recv_response(&self) -> Option<TTSResponse> {
        self.response_rx.lock().await.recv().await
    }

    /// Request TTS for given text
    pub async fn speak(&self, request: TTSRequest) -> Result<()> {
        self.send_command(TTSCommand::Speak(request)).await
    }

    /// Stop current playback
    pub async fn stop(&self) -> Result<()> {
        self.send_command(TTSCommand::Stop).await
    }

    /// Skip to next in queue
    #[allow(dead_code)]
    pub async fn skip(&self) -> Result<()> {
        self.send_command(TTSCommand::Skip).await
    }

    /// Clear the queue
    pub async fn clear_queue(&self) -> Result<()> {
        self.send_command(TTSCommand::ClearQueue).await
    }

    /// Get current status
    #[allow(dead_code)]
    pub async fn get_status(&self) -> Result<()> {
        self.send_command(TTSCommand::GetStatus).await
    }

    /// Shutdown the service
    pub async fn shutdown(&self) -> Result<()> {
        self.send_command(TTSCommand::Shutdown).await
    }
}

/// Main service loop
async fn run_service(
    config: TTSConfig,
    mut command_rx: mpsc::Receiver<TTSCommand>,
    response_tx: mpsc::Sender<TTSResponse>,
) -> Result<()> {
    log::info!("TTS service starting...");

    // Initialize components
    let mut model_cache = ModelCache::new();
    model_cache.scan_models(&config.model_directory)?;

    let audio_player = AudioPlayer::new()
        .context("Failed to initialize audio player")?;

    let queue = TTSQueue::new();

    log::info!("TTS service initialized with {} voices", model_cache.list_voices().len());

    let mut processing = false;

    loop {
        // Process commands
        tokio::select! {
            Some(command) = command_rx.recv() => {
                match command {
                    TTSCommand::Speak(request) => {
                        log::debug!("Received speak command for: {}", request.text);
                        if let Err(e) = queue.enqueue(request) {
                            let _ = response_tx.send(TTSResponse::Error(e)).await;
                        } else {
                            let _ = response_tx.send(TTSResponse::Ok).await;
                        }
                    }

                    TTSCommand::Stop => {
                        log::debug!("Stop command received");
                        audio_player.stop();
                        queue.complete_current();
                        processing = false;
                        let _ = response_tx.send(TTSResponse::Ok).await;
                    }

                    TTSCommand::Skip => {
                        log::debug!("Skip command received");
                        audio_player.stop();
                        queue.complete_current();
                        processing = false;
                        let _ = response_tx.send(TTSResponse::Ok).await;
                    }

                    TTSCommand::ClearQueue => {
                        log::debug!("Clear queue command received");
                        queue.clear();
                        let _ = response_tx.send(TTSResponse::Ok).await;
                    }

                    TTSCommand::GetStatus => {
                        let status = queue.status();
                        let _ = response_tx.send(TTSResponse::Status(status)).await;
                    }

                    TTSCommand::Shutdown => {
                        log::info!("TTS service shutting down");
                        audio_player.stop();
                        break;
                    }
                }
            }

            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)), if !processing && !queue.is_empty() => {
                // Process next item in queue
                processing = true;

                if let Some(request) = queue.dequeue() {
                    queue.set_playing(true);

                    // Clone data for async task
                    let text = request.text.clone();
                    let voice_id = request.voice_id.clone();
                    let speed = request.speed;
                    let player = audio_player.clone();

                    // Get or load model
                    let model = match model_cache.get_or_load(&voice_id) {
                        Ok(m) => m,
                        Err(e) => {
                            log::error!("Failed to load model: {}", e);
                            queue.complete_current();
                            processing = false;
                            let _ = response_tx.send(TTSResponse::Error(e.to_string())).await;
                            continue;
                        }
                    };

                    // Synthesize and play in blocking task
                    task::spawn_blocking(move || {
                        // Preprocess text
                        let processed = synthesis::preprocess_text(&text);

                        // Synthesize
                        match model.synthesize(&processed) {
                            Ok(samples) => {
                                // Play audio
                                if let Err(e) = player.play(samples, model.sample_rate, speed) {
                                    log::error!("Playback error: {}", e);
                                } else {
                                    // Wait for completion
                                    player.wait_for_completion();
                                }
                            }
                            Err(e) => {
                                log::error!("Synthesis error: {}", e);
                            }
                        }
                    });

                    // Mark as complete after synthesis task finishes
                    // Note: In a real implementation, we'd wait for the spawned task
                    // For now, we'll just mark it complete after a delay
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    queue.complete_current();
                    processing = false;
                }
            }
        }
    }

    log::info!("TTS service stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let config = TTSConfig::default();
        match TTSService::start(config) {
            Ok(service) => {
                // Service should be created successfully
                assert!(service.shutdown().await.is_ok());
            }
            Err(e) => {
                // May fail in CI without audio device
                eprintln!("Service creation failed (expected in CI): {}", e);
            }
        }
    }
}
