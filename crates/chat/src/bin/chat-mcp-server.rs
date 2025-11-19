use anyhow::Result;

#[cfg(feature = "mcp")]
use chat::mcp_server::ChatMcpServer;
#[cfg(feature = "mcp")]
use chat::SourcesManager;

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is used for JSON-RPC)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    eprintln!("Chat MCP Server v0.1.0");
    eprintln!("Protocol: Model Context Protocol (MCP)");
    eprintln!("Transport: stdio (JSON-RPC)");
    eprintln!();

    // Create sources manager
    let manager = SourcesManager::new();

    // Note: In a real implementation, this would load configured sources
    // For now, the server will report empty sources until they are configured

    // Create and run server
    let server = ChatMcpServer::new(manager);
    server.run_stdio().await?;

    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() {
    eprintln!("Error: MCP server requires the 'mcp' feature to be enabled");
    eprintln!("Build with: cargo build --features mcp --bin chat-mcp-server");
    std::process::exit(1);
}
