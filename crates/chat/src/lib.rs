pub mod filter_parser;
pub mod matrix_client;
pub mod sources_manager;
pub mod telegram_client;
pub mod telegram_source;
pub mod types;

pub use matrix_client::MatrixChatClient;
pub use sources_manager::SourcesManager;
pub use telegram_client::TelegramChatClient;
pub use telegram_source::TelegramSource;
pub use types::{
    // Legacy types (maintained for backward compatibility)
    Chat, ChatClient, ChatClientConfig, ChatClientId, ChatClientStatus, ChatId, ChatPlatform,
    ChatType, Message, MessageContent, MessageFetchOptions, MessageId, User, UserId,
    // New unified API types
    ChatFilter, ChatPattern, ChatSource, ContentType, MessageFilter, SourceInfo,
};
