use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToolcallSchema {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToolcallRequest {
    pub name: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToolcallResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait Toolcall: Send + Sync {
    fn get_schema(&self) -> ToolcallSchema;
    async fn execute(&self, parameters: Value) -> Result<ToolcallResult>;
}

#[allow(dead_code)]
pub struct ToolcallRegistry {
    tools: HashMap<String, Box<dyn Toolcall>>,
}

#[allow(dead_code)]
impl ToolcallRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Toolcall>) {
        let schema = tool.get_schema();
        self.tools.insert(schema.name, tool);
    }

    pub fn get_schemas(&self) -> Vec<ToolcallSchema> {
        self.tools.values().map(|t| t.get_schema()).collect()
    }

    pub async fn execute(&self, request: ToolcallRequest) -> Result<ToolcallResult> {
        let tool = self
            .tools
            .get(&request.name)
            .ok_or_else(|| anyhow::anyhow!("Toolcall '{}' not found", request.name))?;

        tool.execute(request.parameters).await
    }
}

impl Default for ToolcallRegistry {
    fn default() -> Self {
        Self::new()
    }
}
