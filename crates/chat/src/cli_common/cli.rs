use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Compact single-line format
    Compact,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ChatTypeFilter {
    /// Direct messages
    #[value(name = "dm")]
    DirectMessage,
    /// Group chats
    #[value(name = "group")]
    Group,
    /// Channels
    #[value(name = "channel")]
    Channel,
}
