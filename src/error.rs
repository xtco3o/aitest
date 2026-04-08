use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MCP 协议错误: {0}")]
    Rmcp(#[from] rmcp::RmcpError),

    #[error("IO 错误: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

/// 项目统一的 Result 类型
pub type Result<T> = std::result::Result<T, Error>;
