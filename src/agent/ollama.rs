use super::types::{Agent, AgentConfig, AgentId, AgentStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub model: String,
}

pub struct OllamaAgent {
    config: AgentConfig,
    ollama_config: OllamaConfig,
    status: Arc<RwLock<AgentStatus>>,
    client: reqwest::Client,
    conversation_history: Arc<RwLock<Vec<OllamaMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
}

impl OllamaAgent {
    pub fn new(config: AgentConfig) -> Result<Self> {
        let ollama_config: OllamaConfig = serde_json::from_value(config.config_data.clone())?;

        Ok(Self {
            config,
            ollama_config,
            status: Arc::new(RwLock::new(AgentStatus::Disconnected)),
            client: reqwest::Client::new(),
            conversation_history: Arc::new(RwLock::new(Vec::new())),
        })
    }
}

#[async_trait::async_trait]
impl Agent for OllamaAgent {
    async fn send_message(&self, msg: String) -> Result<()> {
        let mut history = self.conversation_history.write().await;

        history.push(OllamaMessage {
            role: "user".to_string(),
            content: msg,
        });

        let request = OllamaChatRequest {
            model: self.ollama_config.model.clone(),
            messages: history.clone(),
            stream: false,
        };

        let url = format!("{}/api/chat", self.ollama_config.host);
        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API error {}: {}",
                status,
                error_text
            ));
        }

        let chat_response: OllamaChatResponse = response.json().await?;
        history.push(chat_response.message);

        Ok(())
    }

    fn get_status(&self) -> AgentStatus {
        // This is synchronous, so we can't await
        // In a real implementation, we'd need a different approach
        AgentStatus::Connected
    }

    fn get_id(&self) -> AgentId {
        self.config.id
    }

    fn get_config(&self) -> &AgentConfig {
        &self.config
    }

    async fn connect(&mut self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = AgentStatus::Connecting;

        // Test connection by listing models
        let url = format!("{}/api/tags", self.ollama_config.host);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                *status = AgentStatus::Connected;
                Ok(())
            }
            Ok(resp) => {
                let error = format!("Ollama connection failed: {}", resp.status());
                *status = AgentStatus::Error(error.clone());
                Err(anyhow::anyhow!(error))
            }
            Err(e) => {
                let error = format!("Failed to connect to Ollama: {}", e);
                *status = AgentStatus::Error(error.clone());
                Err(anyhow::anyhow!(error))
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = AgentStatus::Disconnected;
        Ok(())
    }
}
