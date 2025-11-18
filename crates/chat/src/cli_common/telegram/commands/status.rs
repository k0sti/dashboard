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

    if session_file.exists() {
        println!("  {}: {}", "Session Status".bold(), "Found".green());
        println!();
        println!("  {}", "Note:".yellow().bold());
        println!("  The Telegram client is not yet fully implemented.");
        println!("  Session validation requires the grammers-client library.");
    } else {
        println!("  {}: {}", "Session Status".bold(), "Not found".yellow());
        println!();
        println!("  Run {} to authenticate", "chat telegram init".cyan());
    }

    Ok(())
}
