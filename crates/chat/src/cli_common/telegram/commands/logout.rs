use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

pub async fn execute() -> Result<()> {
    println!("{}", "Logging out...".bold());

    let session_file = Config::session_file()?;

    if session_file.exists() {
        std::fs::remove_file(&session_file)?;
        println!("{}", format!("Session file deleted: {:?}", session_file).green());
    } else {
        println!("{}", "No session file found.".yellow());
    }

    println!();
    println!("  Run {} to re-authenticate", "chat telegram init".cyan());

    Ok(())
}
