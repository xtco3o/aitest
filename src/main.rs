use directories::ProjectDirs;
use rmcp::ServiceExt;
use std::sync::Arc;
mod error;
mod srv;
mod store;

use tokio::io;

use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME");

    // 获取系统的用户级别缓存目录
    let project_dirs = ProjectDirs::from("", "", pkg_name).expect("无法获取项目目录");

    let cache_dir = project_dirs.cache_dir();
    let index_path = cache_dir.join("index");

    eprintln!("索引保存路径: {:?}", index_path);

    // 初始化 Store
    let store = Arc::new(store::ExperienceStore::open_or_create(index_path)?);

    // 初始化 McpSrv
    let server = srv::McpSrv::new(store);

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
