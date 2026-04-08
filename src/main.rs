use rmcp::ServiceExt;
mod error;
mod srv;

use tokio::io;

use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化模块化后的 Srv
    let server = srv::AideMcpSrv::new();

    // 使用 tokio 的标准输入输出作为传输层
    let transport = (io::stdin(), io::stdout());

    eprintln!(
        "正在启动 MCP 服务 ({} v{})...",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // 启动服务，并将底层错误包装为自定义错误
    server
        .serve(transport)
        .await
        .map_err(|e| error::Error::Other(format!("{:?}", e)))?;

    Ok(())
}
