use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::SourcesManager;

use super::tools::*;
use super::*;

/// MCP JSON-RPC request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// MCP JSON-RPC response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// MCP JSON-RPC error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Error codes
const ERROR_PARSE_ERROR: i32 = -32700;
const ERROR_INVALID_REQUEST: i32 = -32600;
const ERROR_METHOD_NOT_FOUND: i32 = -32601;
const ERROR_INVALID_PARAMS: i32 = -32602;
const ERROR_INTERNAL_ERROR: i32 = -32603;

/// MCP Server implementation
pub struct ChatMcpServer {
    manager: SourcesManager,
}

impl ChatMcpServer {
    /// Create a new MCP server
    pub fn new(manager: SourcesManager) -> Self {
        Self { manager }
    }

    /// Run the server on stdio
    pub async fn run_stdio(&self) -> Result<()> {
        eprintln!("Chat MCP Server starting on stdio...");
        eprintln!("Available tools: list_sources, list_chats, get_messages");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            eprintln!("Received request: {}", line);

            let response = self.handle_request(&line).await;
            let response_json = serde_json::to_string(&response)?;

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;

            eprintln!("Sent response");
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
        // Parse request
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: ERROR_PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        // Handle the method
        match self.handle_method(&request).await {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(e) => {
                let (code, message) = match e.downcast_ref::<MethodError>() {
                    Some(MethodError::MethodNotFound(msg)) => (ERROR_METHOD_NOT_FOUND, msg.clone()),
                    Some(MethodError::InvalidParams(msg)) => (ERROR_INVALID_PARAMS, msg.clone()),
                    None => (ERROR_INTERNAL_ERROR, e.to_string()),
                };

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code,
                        message,
                        data: None,
                    }),
                }
            }
        }
    }

    /// Handle a specific method
    async fn handle_method(&self, request: &JsonRpcRequest) -> Result<Value> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => Err(MethodError::MethodNotFound(format!(
                "Method '{}' not found",
                request.method
            ))
            .into()),
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, _request: &JsonRpcRequest) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "chat-mcp-server",
                "version": "0.1.0"
            }
        }))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": TOOL_LIST_SOURCES,
                    "description": "List all configured chat sources (telegram, signal, whatsapp) with their connection status",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                },
                {
                    "name": TOOL_LIST_CHATS,
                    "description": "List chats from a specific source with optional filtering by name pattern or chat type",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "source": {
                                "type": "string",
                                "description": "Source ID (telegram, signal, whatsapp)"
                            },
                            "name_pattern": {
                                "type": "string",
                                "description": "Filter by name pattern (case-insensitive substring)"
                            },
                            "chat_type": {
                                "type": "string",
                                "description": "Filter by chat type (direct, group, channel)",
                                "enum": ["direct", "group", "channel"]
                            }
                        },
                        "required": ["source"]
                    }
                },
                {
                    "name": TOOL_GET_MESSAGES,
                    "description": "Get messages from a chat with advanced filtering by time, sender, and content",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "source": {
                                "type": "string",
                                "description": "Source ID (telegram, signal, whatsapp). Optional - queries all sources if not specified"
                            },
                            "chat": {
                                "type": "string",
                                "description": "Chat identifier (name, ID, or pattern like 'Antti' or '*' for all)"
                            },
                            "since": {
                                "type": "string",
                                "description": "Messages after this time (e.g., '7d', '2h', '2025-01-15')"
                            },
                            "before": {
                                "type": "string",
                                "description": "Messages before this time"
                            },
                            "sender": {
                                "type": "string",
                                "description": "Filter by sender name or ID"
                            },
                            "search": {
                                "type": "string",
                                "description": "Text search (case-insensitive substring)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Limit number of results (default: 100)"
                            }
                        },
                        "required": ["chat"]
                    }
                }
            ]
        }))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> Result<Value> {
        let params = request.params.as_ref().ok_or_else(|| {
            MethodError::InvalidParams("Missing params for tools/call".to_string())
        })?;

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MethodError::InvalidParams("Missing tool name".to_string()))?;

        let empty_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty_args);

        match tool_name {
            TOOL_LIST_SOURCES => {
                let req: ListSourcesRequest = serde_json::from_value(arguments.clone())?;
                let response = handle_list_sources(req, &self.manager).await?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&response)?
                    }]
                }))
            }
            TOOL_LIST_CHATS => {
                let req: ListChatsRequest = serde_json::from_value(arguments.clone())?;
                let response = handle_list_chats(req, &self.manager).await?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&response)?
                    }]
                }))
            }
            TOOL_GET_MESSAGES => {
                let req: GetMessagesRequest = serde_json::from_value(arguments.clone())?;
                let response = handle_get_messages(req, &self.manager).await?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&response)?
                    }]
                }))
            }
            _ => Err(MethodError::MethodNotFound(format!(
                "Tool '{}' not found",
                tool_name
            ))
            .into()),
        }
    }
}

/// Custom error types for method handling
#[derive(Debug)]
enum MethodError {
    MethodNotFound(String),
    InvalidParams(String),
}

impl std::fmt::Display for MethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodError::MethodNotFound(msg) => write!(f, "Method not found: {}", msg),
            MethodError::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
        }
    }
}

impl std::error::Error for MethodError {}
