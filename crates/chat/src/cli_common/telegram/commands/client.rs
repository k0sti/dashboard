use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::config::Config;

#[cfg(feature = "telegram")]
pub use grammers_client::Client;
#[cfg(feature = "telegram")]
use grammers_mtsender::SenderPool;
#[cfg(feature = "telegram")]
use grammers_session::storages::SqliteSession;

/// Create a Telegram client with the stored session
#[cfg(feature = "telegram")]
pub async fn create_client() -> Result<(Client, JoinHandle<()>)> {
    let config = Config::load()?;

    let api_id = config
        .api_id
        .context("API ID not configured. Run 'chat telegram init'")?;

    // Get session file path
    let session_path = Config::session_file()?;
    let session_path_str = session_path
        .to_str()
        .context("Invalid session path")?;

    // Check if session file exists
    if !session_path.exists() {
        anyhow::bail!("Session not found. Run 'chat telegram init' to authenticate");
    }

    // Load session
    let session = Arc::new(SqliteSession::open(session_path_str)?);

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
pub async fn create_client_unchecked() -> Result<(Client, JoinHandle<()>)> {
    let config = Config::load()?;

    let api_id = config
        .api_id
        .context("API ID not configured. Run 'chat telegram init'")?;

    // Get session file path
    let session_path = Config::session_file()?;
    let session_path_str = session_path
        .to_str()
        .context("Invalid session path")?;

    // Check if session file exists
    if !session_path.exists() {
        anyhow::bail!("Session not found");
    }

    // Load session
    let session = Arc::new(SqliteSession::open(session_path_str)?);

    // Create sender pool and client
    let pool = SenderPool::new(Arc::clone(&session), api_id);
    let client = Client::new(&pool);

    // Start the network runner
    let SenderPool { runner, .. } = pool;
    let runner_handle = tokio::spawn(runner.run());

    Ok((client, runner_handle))
}
