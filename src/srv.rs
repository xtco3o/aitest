use crate::error::{Error, Result};
use crate::store::{Experience, ExperienceStore};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SaveExperienceArgs {
    #[doc = "经验的唯一标识，如果不提供则会自动生成。"]
    pub id: Option<String>,
    #[doc = "经验的标题。"]
    pub title: String,
    #[doc = "经验的内容。"]
    pub content: String,
    #[doc = "经验的标签列表。"]
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SearchExperienceArgs {
    #[doc = "查询字符串。"]
    pub query: String,
    #[doc = "返回结果的数量限制，默认为 5。"]
    pub limit: Option<usize>,
}

#[derive(Clone)]
pub struct McpSrv {
    tool_router: ToolRouter<Self>,
    store: Arc<ExperienceStore>,
}

#[tool_router]
impl McpSrv {
    pub fn new(store: Arc<ExperienceStore>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            store,
        }
    }

    #[tool(description = "保存一条 AI 经验。")]
    async fn save_experience(
        &self,
        Parameters(args): Parameters<SaveExperienceArgs>,
    ) -> Result<String> {
        let id = args.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let exp = Experience {
            id: id.clone(),
            title: args.title,
            content: args.content,
            tags: args.tags.unwrap_or_default(),
            created_at,
        };

        self.store.add_experience(exp).await?;
        Ok(format!("已保存经验，ID: {}", id))
    }

    #[tool(description = "在系统中搜索 AI 经验。")]
    async fn search_experience(
        &self,
        Parameters(args): Parameters<SearchExperienceArgs>,
    ) -> Result<String> {
        let results = self
            .store
            .search(&args.query, args.limit.unwrap_or(5))
            .await?;

        let json_results = sonic_rs::to_string_pretty(&results)
            .map_err(|e| Error::Init(format!("JSON 序列化失败: {}", e)))?;

        Ok(json_results)
    }
}

impl ServerHandler for McpSrv {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> StdResult<CallToolResult, McpError> {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> StdResult<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> StdResult<InitializeResult, McpError> {
        let mut info = InitializeResult::default();
        info.protocol_version = ProtocolVersion::LATEST;
        info.capabilities = ServerCapabilities::default();

        let mut server_info = Implementation::default();
        server_info.name = env!("CARGO_PKG_NAME").to_string();
        server_info.version = env!("CARGO_PKG_VERSION").to_string();

        info.server_info = server_info;
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ExperienceStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_srv_logic() {
        let store = Arc::new(ExperienceStore::open_or_create(":memory:").await.unwrap());
        let srv = McpSrv::new(store);

        // Test save
        let args = SaveExperienceArgs {
            id: Some("test".to_string()),
            title: "标题".to_string(),
            content: "内容".to_string(),
            tags: Some(vec!["tag".to_string()]),
        };
        let res = srv.save_experience(Parameters(args)).await.unwrap();
        assert!(res.contains("test"));

        // Test search
        let search_args = SearchExperienceArgs {
            query: "标题".to_string(),
            limit: Some(1),
        };
        let search_res = srv
            .search_experience(Parameters(search_args))
            .await
            .unwrap();
        assert!(search_res.contains("内容"));
    }
}
