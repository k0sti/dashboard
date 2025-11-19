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
#[path = "../types.rs"]
mod types;
#[path = "../unified_commands/mod.rs"]
mod unified_commands;

#[derive(Parser)]
#[command(name = "chat")]
#[command(about = "Unified CLI for reading messages from Telegram, WhatsApp, and Signal", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress output (quiet mode)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// List all configured chat sources
    #[command(visible_alias = "source")]
    Sources,

    /// List chats from a source
    Chats {
        /// Source ID (telegram, signal, whatsapp)
        source: String,

        /// Filter by name pattern
        #[arg(long)]
        name: Option<String>,

        /// Filter by chat type (direct, group, channel)
        #[arg(long)]
        chat_type: Option<String>,

        /// Output format (text, json, csv, compact)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Get messages with filters
    Messages {
        /// Source and chat filter (format: source:pattern, e.g., "telegram:Antti", "*:*")
        filter: String,

        /// Time range - messages after this time (e.g., "7d", "2h", "2025-01-15")
        #[arg(long)]
        since: Option<String>,

        /// Time range - messages before this time
        #[arg(long)]
        before: Option<String>,

        /// Sender filter (name or ID pattern)
        #[arg(long)]
        sender: Option<String>,

        /// Text search (case-insensitive substring)
        #[arg(long)]
        search: Option<String>,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Output format (text, json, csv, compact)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Telegram commands (legacy, use unified commands instead)
    #[command(hide = false)]
    Telegram {
        #[command(subcommand)]
        command: telegram::TelegramCommand,
    },

    /// WhatsApp commands (not yet implemented)
    #[command(hide = true)]
    Whatsapp {
        #[command(subcommand)]
        command: WhatsAppCommand,
    },

    /// Signal commands (not yet implemented)
    #[command(hide = true)]
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

    match cli.command {
        Command::Sources => unified_commands::sources::execute().await,
        Command::Chats { source, name, chat_type, format } => {
            unified_commands::chats::execute(source, name, chat_type, format).await
        }
        Command::Messages { filter, since, before, sender, search, limit, format } => {
            unified_commands::messages::execute(filter, since, before, sender, search, limit, format).await
        }
        Command::Telegram { command } => telegram::execute(command).await,
        Command::Whatsapp { command: _ } => {
            use colored::Colorize;
            println!("{}", "WhatsApp CLI is not yet implemented.".yellow());
            println!("See the OpenSpec proposal: openspec/changes/add-whatsapp-cli/");
            Ok(())
        }
        Command::Signal { command: _ } => {
            use colored::Colorize;
            println!("{}", "Signal CLI is not yet implemented.".yellow());
            println!("See the OpenSpec proposal: openspec/changes/add-signal-cli/");
            Ok(())
        }
    }
}
