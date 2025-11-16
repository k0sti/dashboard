use super::types::{Toolcall, ToolcallResult, ToolcallSchema};
use anyhow::Result;
use serde_json::Value;
use std::process::Command;
use std::time::Duration;

#[allow(dead_code)]
pub struct ShellToolcall {
    timeout: Duration,
}

#[allow(dead_code)]
impl ShellToolcall {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for ShellToolcall {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Toolcall for ShellToolcall {
    fn get_schema(&self) -> ToolcallSchema {
        ToolcallSchema {
            name: "shell".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory (optional)"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, parameters: Value) -> Result<ToolcallResult> {
        let command_str = parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        let working_dir = parameters
            .get("working_dir")
            .and_then(|v| v.as_str());

        log::info!("Executing shell command: {}", command_str);

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command_str]);
            c
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = tokio::time::timeout(self.timeout, tokio::task::spawn_blocking(move || {
            cmd.output()
        }))
        .await???;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let combined_output = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\nSTDERR:\n{}", stdout, stderr)
        };

        Ok(ToolcallResult {
            success,
            output: combined_output,
            error: if success {
                None
            } else {
                Some(format!("Command failed with exit code: {:?}", output.status.code()))
            },
        })
    }
}
