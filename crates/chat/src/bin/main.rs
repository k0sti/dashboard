use anyhow::Result;
use clap::{Parser, Subcommand};

// Import CLI modules from lib
#[path = "../cli_common/cli.rs"]
mod cli;
#[path = "../cli_common/config.rs"]
mod config;
#[path = "../cli_common/formatters.rs"]
mod formatters;
#[path = "../cli_common/telegram/mod.rs"]
mod telegram;

use cli::*;

#[derive(Parser)]
#[command(name = "chat")]
#[command(about = "Unified CLI for reading messages from Telegram, WhatsApp, and Signal", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    platform: Platform,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress output (quiet mode)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Platform {
    /// Telegram commands
    Telegram {
        #[command(subcommand)]
        command: telegram::TelegramCommand,
    },
    /// WhatsApp commands (not yet implemented)
    Whatsapp {
        #[command(subcommand)]
        command: WhatsAppCommand,
    },
    /// Signal commands (not yet implemented)
    Signal {
        #[command(subcommand)]
        command: SignalCommand,
    },
}

#[derive(Subcommand)]
enum WhatsAppCommand {
    /// Initialize WhatsApp connection
    Init,
    /// Show status (placeholder)
    Status,
}

#[derive(Subcommand)]
enum SignalCommand {
    /// Initialize Signal connection
    Init,
    /// Show status (placeholder)
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else if !cli.quiet {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.platform {
        Platform::Telegram { command } => telegram::execute(command).await,
        Platform::Whatsapp { command } => {
            use colored::Colorize;
            println!("{}", "WhatsApp CLI is not yet implemented.".yellow());
            println!("See the OpenSpec proposal: openspec/changes/add-whatsapp-cli/");
            Ok(())
        }
        Platform::Signal { command } => {
            use colored::Colorize;
            println!("{}", "Signal CLI is not yet implemented.".yellow());
            println!("See the OpenSpec proposal: openspec/changes/add-signal-cli/");
            Ok(())
        }
    }
}
