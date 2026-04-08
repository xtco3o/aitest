use rmcp::{tool, tool_router, transport::stdio::StdioServerTransport, Server};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use anyhow::Result;

/// Arguments for the echo tool.
#[derive(Deserialize, Serialize, JsonSchema, Debug)]
struct EchoArgs {
    /// The message to echo back.
    message: String,
}

/// The state of our MCP server.
#[derive(Default)]
struct AideMcpServer;

#[tool_router]
impl AideMcpServer {
    /// Echoes back the message provided.
    #[tool(description = "Returns the message provided by the user.")]
    async fn echo(&self, args: EchoArgs) -> Result<String> {
        eprintln!("Echoing back: {}", args.message);
        Ok(format!("Echo: {}", args.message))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 使用环境变量获取 Cargo.toml 中的名称和版本
    let server = Server::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    
    // Use stdio transport for local integration with Claude.
    let transport = StdioServerTransport::new();
    
    eprintln!("Starting MCP server...");
    
    // Run the server.
    server.run(transport, AideMcpServer::default()).await?;
    
    Ok(())
}
