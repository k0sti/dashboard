use crate::agent::AgentConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub agents: Vec<AgentConfig>,
}

impl AppConfig {
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("agent-dashboard");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("agents.json"))
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(config_file)?;
        let config: Self = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(config_file, contents)?;
        Ok(())
    }

    pub fn add_agent(&mut self, agent: AgentConfig) {
        self.agents.push(agent);
    }

    pub fn remove_agent(&mut self, id: &crate::agent::AgentId) {
        self.agents.retain(|a| a.id != *id);
    }
}
