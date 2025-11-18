pub mod matrix_client;
pub mod telegram_client;
pub mod types;

pub use matrix_client::MatrixChatClient;
pub use telegram_client::TelegramChatClient;
pub use types::{
    Chat, ChatClient, ChatClientConfig, ChatClientId, ChatClientStatus, ChatId, ChatPlatform,
    ChatType, Message, MessageContent, MessageFetchOptions, MessageId, User, UserId,
};
