use rmcp::model::{Content, IntoContents};
use std::result::Result as StdResult;
use thiserror::Error;
use tokio::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MCP 协议错误: {0}")]
    Rmcp(#[from] Box<rmcp::RmcpError>),

    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

/// 实现 IntoContents 以便 rmcp 可以将错误作为响应内容发送
impl IntoContents for Error {
    fn into_contents(self) -> Vec<Content> {
        vec![Content::text(self.to_string())]
    }
}

/// 项目统一的 Result 类型
pub type Result<T> = StdResult<T, Error>;
