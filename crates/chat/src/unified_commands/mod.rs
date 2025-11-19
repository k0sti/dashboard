use anyhow::Result;
use clap::Subcommand;

pub mod sources;
pub mod chats;
pub mod messages;

#[derive(Subcommand)]
pub enum UnifiedCommand {
    /// List all configured chat sources
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
}

pub async fn execute(command: UnifiedCommand) -> Result<()> {
    match command {
        UnifiedCommand::Sources => sources::execute().await,
        UnifiedCommand::Chats {
            source,
            name,
            chat_type,
            format,
        } => chats::execute(source, name, chat_type, format).await,
        UnifiedCommand::Messages {
            filter,
            since,
            before,
            sender,
            search,
            limit,
            format,
        } => messages::execute(filter, since, before, sender, search, limit, format).await,
    }
}
