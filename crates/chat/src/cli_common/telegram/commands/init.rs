use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::Config;

pub async fn execute(
    api_id: Option<i32>,
    api_hash: Option<String>,
    phone: Option<String>,
) -> Result<()> {
    println!("{}", "Initializing Telegram connection...".bold());

    let mut config = Config::load()?;

    // Get or prompt for API credentials
    let api_id = api_id
        .or(config.api_id)
        .context("API ID is required. Set via --api-id, config, or TELEGRAM_API_ID env var")?;

    let api_hash = api_hash
        .or(config.api_hash.clone())
        .context("API Hash is required. Set via --api-hash, config, or TELEGRAM_API_HASH env var")?;

    let phone = phone
        .or(config.phone.clone())
        .context("Phone number is required. Set via --phone flag")?;

    // Save configuration
    config.api_id = Some(api_id);
    config.api_hash = Some(api_hash.clone());
    config.phone = Some(phone.clone());
    config.save()?;

    println!("{}", "Configuration saved.".green());

    #[cfg(feature = "telegram")]
    {
        telegram_auth(api_id, &api_hash, &phone).await?;
    }

    #[cfg(not(feature = "telegram"))]
    {
        println!("\n{}", "Note:".yellow().bold());
        println!("  The telegram feature is not enabled.");
        println!("  Build with: cargo build --features telegram");
    }

    Ok(())
}

#[cfg(feature = "telegram")]
async fn telegram_auth(api_id: i32, api_hash: &str, phone: &str) -> Result<()> {
    use std::io::{self, Write};
    use std::sync::Arc;
    use grammers_client::{Client, SignInError};
    use grammers_mtsender::SenderPool;
    use grammers_session::storages::MemorySession;

    println!("{}", "Connecting to Telegram...".bold());

    // Note: Using MemorySession (session won't persist across restarts)
    // This avoids SQLite conflicts with WhatsApp storage
    let session = Arc::new(MemorySession::default());

    // Create sender pool and client
    println!("{}", "Initializing Telegram client...".bold());
    let pool = SenderPool::new(Arc::clone(&session), api_id);
    let client = Client::new(&pool);

    // Start the network runner
    let SenderPool { runner, .. } = pool;
    let runner_handle = tokio::spawn(runner.run());

    // Check if already signed in (unlikely with MemorySession, but check anyway)
    if client.is_authorized().await? {
        println!("{}", "✓ Already signed in!".green().bold());

        // Get user info
        match client.get_me().await {
            Ok(me) => {
                println!("  User: {}", me.first_name().unwrap_or("Unknown"));
                if let Some(username) = me.username() {
                    println!("  Username: @{}", username);
                }
            }
            Err(e) => {
                println!("  {}: {}", "Warning".yellow(), e);
            }
        }

        runner_handle.abort();
        return Ok(());
    }

    println!("{}", "Requesting authentication code...".bold());

    // Request login code
    let token = client
        .request_login_code(phone, api_hash)
        .await
        .context("Failed to request login code")?;

    // Prompt for auth code
    print!("Enter the code you received: ");
    io::stdout().flush()?;
    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let code = code.trim();

    println!("{}", "Signing in...".bold());

    // Sign in
    match client.sign_in(&token, code).await {
        Ok(_) => {
            println!("{}", "✓ Successfully signed in!".green().bold());
            println!("  Note: Session uses in-memory storage (won't persist across restarts)");

            // Get user info
            match client.get_me().await {
                Ok(me) => {
                    println!("  User: {}", me.first_name().unwrap_or("Unknown"));
                    if let Some(username) = me.username() {
                        println!("  Username: @{}", username);
                    }
                }
                Err(e) => {
                    println!("  {}: {}", "Warning".yellow(), e);
                }
            }
        }
        Err(SignInError::PasswordRequired(password_token)) => {
            // 2FA is enabled
            print!("Two-factor authentication enabled.");
            if let Some(hint) = password_token.hint() {
                print!(" Hint: {}", hint);
            }
            println!();
            print!("Enter your password: ");
            io::stdout().flush()?;
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            client
                .check_password(password_token, password)
                .await
                .context("Failed to sign in with password")?;

            println!("{}", "✓ Successfully signed in!".green().bold());
            println!("  Note: Session uses in-memory storage (won't persist across restarts)");
        }
        Err(e) => {
            runner_handle.abort();
            return Err(anyhow::anyhow!("Failed to sign in: {}", e));
        }
    }

    // Stop the runner
    runner_handle.abort();

    Ok(())
}
