use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

pub async fn execute() -> Result<()> {
    let config = Config::load()?;
    let session_file = Config::session_file()?;

    println!("{}", "Telegram Connection Status".bold());
    println!();

    if config.api_id.is_some() && config.api_hash.is_some() {
        println!("  {}: {}", "Configuration".bold(), "Present".green());
        println!("  {}: {:?}", "API ID", config.api_id.unwrap());
        println!("  {}: {}", "API Hash", "***".dimmed());

        if let Some(phone) = &config.phone {
            println!("  {}: {}", "Phone", phone);
        }
    } else {
        println!("  {}: {}", "Configuration".bold(), "Not configured".red());
        println!();
        println!("  Run {} to configure", "chat telegram init".cyan());
        return Ok(());
    }

    println!();
    println!("  {}: {:?}", "Session File", session_file);

    if !session_file.exists() {
        println!("  {}: {}", "Session Status".bold(), "Not found".yellow());
        println!();
        println!("  Run {} to authenticate", "chat telegram init".cyan());
        return Ok(());
    }

    println!("  {}: {}", "Session Status".bold(), "Found".green());

    // Try to connect and check authorization
    #[cfg(feature = "telegram")]
    {
        use super::client;

        println!();
        println!("  {}", "Checking connection...".dimmed());

        match client::create_client().await {
            Ok((client, runner_handle)) => {
                println!("  {}: {}", "Connection".bold(), "Authorized".green());

                // Get user info
                match client.get_me().await {
                    Ok(me) => {
                        println!();
                        println!("  {}", "Account Information:".bold());
                        println!("    {}: {}", "Name", me.first_name().unwrap_or("Unknown"));
                        if let Some(last_name) = me.last_name() {
                            println!("    {}: {}", "Last Name", last_name);
                        }
                        if let Some(username) = me.username() {
                            println!("    {}: @{}", "Username", username);
                        }
                        println!("    {}: {}", "User ID", me.raw.id());
                    }
                    Err(e) => {
                        println!("  {}: Failed to get user info: {}", "Warning".yellow(), e);
                    }
                }

                runner_handle.abort();
            }
            Err(e) => {
                println!("  {}: {}", "Connection".bold(), "Failed".red());
                println!("  {}: {}", "Error".red(), e);
                println!();
                println!("  Run {} to re-authenticate", "chat telegram init".cyan());
            }
        }
    }

    #[cfg(not(feature = "telegram"))]
    {
        println!();
        println!("  {}", "Note:".yellow().bold());
        println!("  The telegram feature is not enabled.");
        println!("  Build with: cargo build --features telegram");
    }

    Ok(())
}
