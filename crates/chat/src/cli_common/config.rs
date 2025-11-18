use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_id: Option<i32>,
    pub api_hash: Option<String>,
    pub phone: Option<String>,
    pub session_path: Option<String>,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("chat")
            .join("telegram");

        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir)
    }

    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn session_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("session.dat"))
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&config_file)
            .context("Failed to read config file")?;

        toml::from_str(&contents).context("Failed to parse config file")
    }

    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(&config_file, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "api_id" => {
                self.api_id = Some(value.parse()
                    .context("Invalid api_id: must be an integer")?);
            }
            "api_hash" => {
                self.api_hash = Some(value.to_string());
            }
            "phone" => {
                self.phone = Some(value.to_string());
            }
            "session_path" => {
                self.session_path = Some(value.to_string());
            }
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "api_id" => self.api_id.map(|v| v.to_string()),
            "api_hash" => self.api_hash.clone(),
            "phone" => self.phone.clone(),
            "session_path" => self.session_path.clone(),
            _ => None,
        }
    }
}
