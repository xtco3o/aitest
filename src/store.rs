use crate::error::{Error, Result};
use jieba_rs::Jieba;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use turso::{Builder, Connection};

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
            .ok_or_else(|| Error::Init("无效的数据库路径".to_string()))?;
        let db = Builder::new_local(path_str)
            .experimental_index_method(true)
            .build()
            .await?;
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
        // 这一步是关键，它使用了 Turso 新引擎 Limbo 的原生全文搜索功能
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

        // 使用元组作为参数
        self.conn
            .execute(
                "INSERT INTO experiences (id, title, content, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                (exp.id, tokenized_title, tokenized_content, tags_json, exp.created_at),
            )
            .await?;
        Ok(())
    }

    pub async fn search(&self, query_str: &str, limit: usize) -> Result<Vec<Experience>> {
        let tokenized_query = self.tokenize(query_str);

        // 使用 Turso 原生 FTS 函数 fts_match 和 fts_score
        let mut rows = self.conn
            .query(
                "SELECT id, title, content, tags, created_at, fts_score(title, content, ?1) as score 
                 FROM experiences 
                 WHERE fts_match(title, content, ?1) 
                 ORDER BY score DESC 
                 LIMIT ?2",
                (tokenized_query, limit as i64),
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
