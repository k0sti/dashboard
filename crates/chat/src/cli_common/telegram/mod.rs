use anyhow::Result;
use clap::Subcommand;

pub mod commands;
use super::cli::*;
use commands::*;

#[derive(Subcommand)]
pub enum TelegramCommand {
    /// Initialize and authenticate Telegram connection
    Init {
        /// API ID from my.telegram.org
        #[arg(long, env = "TELEGRAM_API_ID")]
        api_id: Option<i32>,

        /// API Hash from my.telegram.org
        #[arg(long, env = "TELEGRAM_API_HASH")]
        api_hash: Option<String>,

        /// Phone number (with country code)
        #[arg(long)]
        phone: Option<String>,
    },

    /// Check connection status
    Status,

    /// List all chats and groups
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Filter by chat type
        #[arg(long)]
        r#type: Option<ChatTypeFilter>,
    },

    /// Get messages from a chat
    Get {
        /// Chat name or ID
        chat: Option<String>,

        /// Chat ID (alternative to chat name)
        #[arg(long)]
        id: Option<String>,

        /// Maximum number of messages to retrieve
        #[arg(short, long, default_value = "100")]
        limit: usize,

        /// Get messages since this timestamp or relative time (e.g., "2 days ago")
        #[arg(long)]
        since: Option<String>,

        /// Get messages before this timestamp
        #[arg(long)]
        before: Option<String>,

        /// Alias for --since
        #[arg(long)]
        after: Option<String>,

        /// Filter by sender username or ID
        #[arg(long)]
        sender: Option<String>,

        /// Filter by message type
        #[arg(long)]
        r#type: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Write output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Watch for new messages in real-time
    Watch {
        /// Chat name or ID (omit to watch all chats)
        chat: Option<String>,

        /// Watch all chats
        #[arg(long)]
        all: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Export messages to a file
    Export {
        /// Chat name or ID
        chat: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export messages since this timestamp or relative time
        #[arg(long)]
        since: Option<String>,

        /// Export messages before this timestamp
        #[arg(long)]
        before: Option<String>,

        /// Maximum number of messages to export
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Search messages by text content
    Search {
        /// Search term
        term: String,

        /// Chat name or ID (omit for --all)
        chat: Option<String>,

        /// Search all chats
        #[arg(long)]
        all: bool,

        /// Case-insensitive search
        #[arg(long)]
        ignore_case: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Get detailed information about a chat
    Info {
        /// Chat name or ID
        chat: Option<String>,

        /// Chat ID (alternative)
        #[arg(long)]
        id: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Logout and clear session
    Logout,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// List all configuration values
    List,
}

pub async fn execute(command: TelegramCommand) -> Result<()> {
    match command {
        TelegramCommand::Init {
            api_id,
            api_hash,
            phone,
        } => init::execute(api_id, api_hash, phone).await,

        TelegramCommand::Status => status::execute().await,

        TelegramCommand::List { format, r#type } => list::execute(format, r#type).await,

        TelegramCommand::Get {
            chat,
            id,
            limit,
            since,
            before,
            after,
            sender,
            r#type,
            format,
            output,
        } => {
            get::execute(chat, id, limit, since, before, after, sender, r#type, format, output)
                .await
        }

        TelegramCommand::Watch { chat, all, format } => watch::execute(chat, all, format).await,

        TelegramCommand::Export {
            chat,
            format,
            output,
            since,
            before,
            limit,
        } => export::execute(chat, format, output, since, before, limit).await,

        TelegramCommand::Search {
            term,
            chat,
            all,
            ignore_case,
            format,
        } => search::execute(chat, term, all, ignore_case, format).await,

        TelegramCommand::Info { chat, id, format } => info::execute(chat, id, format).await,

        TelegramCommand::Config { action } => match action {
            ConfigAction::Set { key, value } => config_cmd::set(key, value).await,
            ConfigAction::Get { key } => config_cmd::get(key).await,
            ConfigAction::List => config_cmd::list().await,
        },

        TelegramCommand::Logout => logout::execute().await,
    }
}
