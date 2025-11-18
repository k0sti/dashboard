use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

pub async fn set(key: String, value: String) -> Result<()> {
    let mut config = Config::load()?;
    config.set(&key, &value)?;
    config.save()?;

    println!("{}", format!("Configuration updated: {} = {}", key, value).green());

    Ok(())
}

pub async fn get(key: String) -> Result<()> {
    let config = Config::load()?;

    match config.get(&key) {
        Some(value) => {
            // Mask sensitive values
            let display_value = if key.contains("hash") || key.contains("password") {
                "***".to_string()
            } else {
                value
            };
            println!("{}: {}", key.bold(), display_value);
        }
        None => {
            println!("{}", format!("Configuration key '{}' not found", key).red());
        }
    }

    Ok(())
}

pub async fn list() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Configuration:".bold());
    println!();

    if let Some(api_id) = config.api_id {
        println!("  {}: {}", "api_id".cyan(), api_id);
    }

    if config.api_hash.is_some() {
        println!("  {}: {}", "api_hash".cyan(), "***".dimmed());
    }

    if let Some(phone) = config.phone {
        println!("  {}: {}", "phone".cyan(), phone);
    }

    if let Some(session_path) = config.session_path {
        println!("  {}: {}", "session_path".cyan(), session_path);
    }

    Ok(())
}
