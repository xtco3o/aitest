use rmcp::model::{Content, IntoContents};
use std::result::Result as StdResult;
use thiserror::Error;
use tokio::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MCP 错误: {0:?}")]
    Rmcp(#[from] rmcp::ErrorData),

    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("初始化或逻辑错误: {0}")]
    Init(String),
}

impl From<turso::Error> for Error {
    fn from(err: turso::Error) -> Self {
        Error::Database(format!("{:?}", err))
    }
}

/// 实现 IntoContents 以便 rmcp 可以将错误作为响应内容发送
impl IntoContents for Error {
    fn into_contents(self) -> Vec<Content> {
        vec![Content::text(self.to_string())]
    }
}

/// 项目统一的 Result 类型
pub type Result<T> = StdResult<T, Error>;
