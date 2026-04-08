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

    let url = std::env::var("TURSO_DATABASE_URL").ok();
    let token = std::env::var("TURSO_AUTH_TOKEN").ok();

    // 初始化 Store
    let store = if let (Some(url), Some(token)) = (url, token) {
        eprintln!("正在连接到 Turso 远程数据库...");
        Arc::new(ExperienceStore::open_remote(url, token).await?)
    } else {
        eprintln!("数据库保存路径: {:?}", index_path);
        Arc::new(ExperienceStore::open_or_create(index_path).await?)
    };

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
