use rmcp::{tool, tool_router, RoleServer, ServerHandler, RmcpError};
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, 
    InitializeResult, Implementation, ProtocolVersion, ServerCapabilities
};
use rmcp::service::RequestContext;
use crate::error::Result;
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
pub struct EchoArgs {
    pub message: String,
}

#[derive(Clone)]
pub struct AideMcpSrv {
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl AideMcpSrv {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "根据输入消息返回相同内容的问候。")]
    async fn echo(&self, Parameters(args): Parameters<EchoArgs>) -> Result<String> {
        eprintln!("收到消息: {}", args.message);
        Ok(format!("Echo: {}", args.message))
    }
}

impl ServerHandler for AideMcpSrv {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, RmcpError> {
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }
    
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: None,
        }
    }
}
