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
async fn telegram_auth(_api_id: i32, _api_hash: &str, _phone: &str) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "Note:".yellow().bold());
    println!("  Telegram client integration is ready but not yet implemented.");
    println!("  The grammers-client v0.8 API requires:");
    println!();
    println!("  1. Create a SenderPool");
    println!("  2. Use SqliteSession::load_file_or_create() for session storage");
    println!("  3. Create Client with Client::new(sender_pool)");
    println!("  4. Implement authentication flow with:");
    println!("     - client.is_authorized()");
    println!("     - client.request_login_code()");
    println!("     - client.sign_in()");
    println!();
    println!("{}", "Resources:".bold());
    println!("  - grammers examples: https://github.com/Lonami/grammers/tree/master/examples");
    println!("  - API docs: https://docs.rs/grammers-client/0.8.1/");
    println!();
    println!("{}", "Session file location:".bold());
    println!("  {}", Config::session_file()?.display());

    Ok(())
}
