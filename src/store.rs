use crate::error::Result;
use jieba_rs::Jieba;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use turso::{Builder, Connection, Database, params::IntoValue};

#[derive(Serialize, Deserialize, Debug)]
pub struct Experience {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: i64,
}

pub struct ExperienceStore {
    conn: Connection,
    jieba: Arc<Jieba>,
}

impl ExperienceStore {
    pub async fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| crate::error::Error::Init("无效的数据库路径".to_string()))?;
        let db = Builder::new_local(path_str).build().await?;
        Self::init_with_db(db).await
    }

    pub async fn open_remote(path: String, url: String, token: String) -> Result<Self> {
        // 使用嵌入式副本 (Embedded Replica) 是 Turso 推荐的 Rust 使用方式
        let db = Builder::new_remote_replica(&path, &url, &token)
            .build()
            .await?;
        // 同步一次
        db.sync().await?;
        Self::init_with_db(db).await
    }

    async fn init_with_db(db: Database) -> Result<Self> {
        let conn = db.connect()?;

        // 初始化表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS experiences (
                id TEXT PRIMARY KEY,
                title TEXT,
                content TEXT,
                tags TEXT,
                created_at INTEGER
            )",
            (),
        )
        .await?;

        // 使用 Turso 原生 FTS 索引 (基于 Tantivy)
        // 这是 Limbo/Turso 的原生功能，语法更加简洁
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_exp_fts ON experiences USING fts(title, content)",
            (),
        )
        .await?;

        Ok(Self {
            conn,
            jieba: Arc::new(Jieba::new()),
        })
    }

    /// 对中文文本进行分词，以便原生 FTS 搜索
    fn tokenize(&self, text: &str) -> String {
        self.jieba.cut(text, false).join(" ")
    }

    pub async fn add_experience(&self, exp: Experience) -> Result<()> {
        let tags_json = serde_json::to_string(&exp.tags).unwrap_or_default();

        // 预分词处理
        let tokenized_title = self.tokenize(&exp.title);
        let tokenized_content = self.tokenize(&exp.content);

        // 使用明确的参数元组以避免类型推导歧义
        let params: [Box<dyn IntoValue>; 5] = [
            Box::new(exp.id),
            Box::new(tokenized_title),
            Box::new(tokenized_content),
            Box::new(tags_json),
            Box::new(exp.created_at),
        ];

        self.conn
            .execute(
                "INSERT INTO experiences (id, title, content, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params,
            )
            .await?;
        Ok(())
    }

    pub async fn search(&self, query_str: &str, limit: usize) -> Result<Vec<Experience>> {
        let tokenized_query = self.tokenize(query_str);

        // 使用 Turso 原生 FTS 函数 fts_match 和 fts_score
        let params: [Box<dyn IntoValue>; 2] = [Box::new(tokenized_query), Box::new(limit as i64)];

        let mut rows = self.conn
            .query(
                "SELECT id, title, content, tags, created_at, fts_score(title, content, ?1) as score 
                 FROM experiences 
                 WHERE fts_match(title, content, ?1) 
                 ORDER BY score DESC 
                 LIMIT ?2",
                params,
            )
            .await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let tags_json: String = row.get(3)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            results.push(Experience {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags,
                created_at: row.get(4)?,
            });
        }

        Ok(results)
    }
}
