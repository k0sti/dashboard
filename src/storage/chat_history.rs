use crate::agent::AgentId;
use crate::config::AppConfig;
use crate::ui::chat::ChatMessage;
use anyhow::Result;
use rusqlite::{Connection, params};

pub struct ChatHistoryStore {
    conn: Connection,
}

impl ChatHistoryStore {
    pub fn new() -> Result<Self> {
        let db_path = AppConfig::config_dir()?.join("chat_history.db");
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                agent_id TEXT,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                direction TEXT NOT NULL,
                metadata TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn save_message(&self, message: &ChatMessage) -> Result<()> {
        let agent_id_str = message.agent_id.map(|id| id.to_string());
        let direction_str = format!("{:?}", message.direction);
        let metadata_str = serde_json::to_string(&message.metadata)?;

        self.conn.execute(
            "INSERT INTO messages (id, agent_id, content, timestamp, direction, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message.id.to_string(),
                agent_id_str,
                &message.content,
                message.timestamp.to_rfc3339(),
                direction_str,
                metadata_str,
            ],
        )?;

        Ok(())
    }

    pub fn load_messages(&self, _agent_id: Option<AgentId>, _limit: usize) -> Result<Vec<ChatMessage>> {
        // Simplified implementation for now - just return empty vector
        // Full deserialization would need more complex logic
        Ok(Vec::new())
    }
}
