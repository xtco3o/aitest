use aitest::error::{Error, Result};
use aitest::srv::McpSrv;
use aitest::store::ExperienceStore;
use directories::ProjectDirs;
use std::sync::Arc;
use tokio::io;

#[tokio::main]
async fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME");

    // 获取系统的用户级别缓存目录
    let project_dirs = ProjectDirs::from("", "", pkg_name).expect("无法获取项目目录");

    let cache_dir = project_dirs.cache_dir();
    let index_path = cache_dir.join("index.db");

    // 目前该版本的 turso crate (Limbo) 侧重于本地高性能存储
    // 我们优先使用本地模式以发挥其 Native FTS (基于 Tantivy) 的优势
    eprintln!("使用 Turso (Limbo) 本地数据库: {:?}", index_path);
    let store = Arc::new(ExperienceStore::open_or_create(index_path).await?);

    // 初始化 McpSrv
    let server = McpSrv::new(store);

    // 使用 tokio 的标准输入输出作为传输层
    let transport = (io::stdin(), io::stdout());

    eprintln!(
        "正在启动 MCP 服务 ({} v{})...",
        pkg_name,
        env!("CARGO_PKG_VERSION")
    );

    // 启动服务
    rmcp::serve_server(server, transport)
        .await
        .map_err(|e| Error::Init(format!("{:?}", e)))?;

    Ok(())
}
