use anyhow::Result;
use colored::Colorize;

use chat::SourcesManager;

pub async fn execute() -> Result<()> {
    println!("{}", "Listing configured chat sources...".dimmed());

    // Create sources manager
    let manager = SourcesManager::new();

    // Note: In a real implementation, this would load sources from configuration
    // For now, we just show what sources are registered

    let sources = manager.list_sources()?;

    if sources.is_empty() {
        println!();
        println!("{}", "No sources configured.".yellow());
        println!();
        println!("To configure a source, use:");
        println!("  {} - Initialize Telegram", "chat telegram init".cyan());
        println!("  {} - Initialize Signal (not yet implemented)", "chat signal init".dimmed());
        println!("  {} - Initialize WhatsApp (not yet implemented)", "chat whatsapp init".dimmed());
        return Ok(());
    }

    println!();
    println!("{}", "Configured Sources:".bold());
    println!();

    for source in sources {
        let status = if source.is_connected {
            "Connected".green()
        } else {
            "Disconnected".red()
        };

        println!("  {} {} - {}",
            "•".cyan(),
            source.name.bold(),
            status
        );
        println!("    {}: {}", "ID".dimmed(), source.id);
    }

    println!();

    Ok(())
}
