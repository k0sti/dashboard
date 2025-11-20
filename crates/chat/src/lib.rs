pub mod filter_parser;
pub mod matrix_client;
#[cfg(feature = "mcp")]
pub mod mcp_server;
pub mod sources_manager;
#[cfg(feature = "telegram")]
pub mod telegram_client;
#[cfg(feature = "telegram")]
pub mod telegram_source;
pub mod types;
#[cfg(feature = "whatsapp")]
pub mod whatsapp_source;

pub use matrix_client::MatrixChatClient;
pub use sources_manager::SourcesManager;
#[cfg(feature = "telegram")]
pub use telegram_client::TelegramChatClient;
#[cfg(feature = "telegram")]
pub use telegram_source::TelegramSource;
#[cfg(feature = "whatsapp")]
pub use whatsapp_source::{WhatsAppSource, WhatsAppConfig};
pub use types::{
    // Legacy types (maintained for backward compatibility)
    Chat, ChatClient, ChatClientConfig, ChatClientId, ChatClientStatus, ChatId, ChatPlatform,
    ChatType, Message, MessageContent, MessageFetchOptions, MessageId, User, UserId,
    // New unified API types
    ChatFilter, ChatPattern, ChatSource, ContentType, MessageFilter, SourceInfo,
};
