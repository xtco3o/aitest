use aitest::srv::{McpSrv, SaveExperienceArgs, SearchExperienceArgs};
use aitest::store::ExperienceStore;
use rmcp::model::{CallToolRequestParams, InitializeRequestParams, Content};
use rmcp::service::RequestContext;
use rmcp::{ServerHandler, RoleServer};
use std::sync::Arc;
use tokio;
use serde_json::{json, Value};

#[tokio::test]
async fn test_mcp_flow() {
    // 1. Setup in-memory store
    let store = Arc::new(ExperienceStore::open_or_create(":memory:").await.unwrap());
    let srv = McpSrv::new(store);
    let context = RequestContext::<RoleServer>::new(RoleServer, Default::default());

    // 2. Test Initialize
    let init_params = InitializeRequestParams::default();
    let init_res = srv.initialize(init_params, context.clone()).await.unwrap();
    assert_eq!(init_res.server_info.name, "aitest");

    // 3. Test List Tools
    let tools = srv.list_tools(None, context.clone()).await.unwrap();
    assert!(tools.tools.iter().any(|t| t.name == "save_experience"));
    assert!(tools.tools.iter().any(|t| t.name == "search_experience"));

    // 4. Test Save Experience
    let save_args = SaveExperienceArgs {
        id: Some("test-id".to_string()),
        title: "Rust 编程经验".to_string(),
        content: "使用 Turso 和 FTS5 实现高效搜索。".to_string(),
        tags: Some(vec!["rust".to_string(), "database".to_string()]),
    };
    
    // CallToolRequestParams is non-exhaustive, but it has fields we can set.
    // We should use what's available. If it's truly restricted, we might need to find a builder or use a specific constructor.
    // Most rmcp models have a default or a way to construct them.
    let mut save_params = CallToolRequestParams::default();
    save_params.name = "save_experience".into();
    let args_value = serde_json::to_value(save_args).unwrap();
    if let Value::Object(map) = args_value {
        save_params.arguments = Some(map);
    }

    let save_res = srv.call_tool(save_params, context.clone()).await.unwrap();
    assert!(save_res.is_error.unwrap_or(false) == false);

    // 5. Test Search Experience
    let search_args = SearchExperienceArgs {
        query: "Turso 搜索".to_string(),
        limit: Some(1),
    };
    
    let mut search_params = CallToolRequestParams::default();
    search_params.name = "search_experience".into();
    let search_args_value = serde_json::to_value(search_args).unwrap();
    if let Value::Object(map) = search_args_value {
        search_params.arguments = Some(map);
    }

    let search_res = srv.call_tool(search_params, context.clone()).await.unwrap();
    assert!(search_res.is_error.unwrap_or(false) == false);
    
    // Verify the search result
    let content = &search_res.content[0];
    // rmcp::model::Content might be a wrapper or an enum. 
    // Based on previous error, Content might be Annotated<RawContent> or similar.
    // Let's try to match on what it likely is based on rmcp source/docs.
    // Usually it has a way to get text.
    let json_text = serde_json::to_string(content).unwrap();
    assert!(json_text.contains("Rust 编程经验"));
}
