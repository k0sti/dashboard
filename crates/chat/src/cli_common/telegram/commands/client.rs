use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::config::Config;

#[cfg(feature = "telegram")]
pub use grammers_client::Client;
#[cfg(feature = "telegram")]
use grammers_mtsender::SenderPool;
#[cfg(feature = "telegram")]
use grammers_session::storages::MemorySession;

/// Create a Telegram client with the stored session
#[cfg(feature = "telegram")]
pub async fn create_client() -> Result<(Client, JoinHandle<()>)> {
    let config = Config::load()?;

    let api_id = config
        .api_id
        .context("API ID not configured. Run 'chat telegram init'")?;

    // Note: Using MemorySession (session won't persist across restarts)
    // This avoids SQLite conflicts with WhatsApp storage
    let session = Arc::new(MemorySession::default());

    // Create sender pool and client
    let pool = SenderPool::new(Arc::clone(&session), api_id);
    let client = Client::new(&pool);

    // Start the network runner
    let SenderPool { runner, .. } = pool;
    let runner_handle = tokio::spawn(runner.run());

    // Check if authorized
    if !client.is_authorized().await? {
        anyhow::bail!("Not authenticated. Run 'chat telegram init' to authenticate");
    }

    Ok((client, runner_handle))
}

/// Create a client without checking authorization (for logout, etc.)
#[cfg(feature = "telegram")]
#[allow(dead_code)]
pub async fn create_client_unchecked() -> Result<(Client, JoinHandle<()>)> {
    let config = Config::load()?;

    let api_id = config
        .api_id
        .context("API ID not configured. Run 'chat telegram init'")?;

    // Note: Using MemorySession (session won't persist across restarts)
    // This avoids SQLite conflicts with WhatsApp storage
    let session = Arc::new(MemorySession::default());

    // Create sender pool and client
    let pool = SenderPool::new(Arc::clone(&session), api_id);
    let client = Client::new(&pool);

    // Start the network runner
    let SenderPool { runner, .. } = pool;
    let runner_handle = tokio::spawn(runner.run());

    Ok((client, runner_handle))
}
