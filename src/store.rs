use crate::error::Result;
use jieba_rs::Jieba;
use libsql::{Connection, Database, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

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
        let db = libsql::Builder::new_local(path.as_ref()).build().await?;
        Self::init_with_db(db).await
    }

    pub async fn open_remote(url: String, token: String) -> Result<Self> {
        let db = libsql::Builder::new_remote(url, token).build().await?;
        Self::init_with_db(db).await
    }

    async fn init_with_db(db: Database) -> Result<Self> {
        let conn = db.connect()?;

        // 初始化表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS experiences (
                id TEXT PRIMARY KEY,
                title TEXT,
                content TEXT,
                tags TEXT,
                created_at INTEGER
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS experiences_fts USING fts5(
                id UNINDEXED,
                title,
                content,
                content='experiences',
                content_rowid='rowid'
            );
            -- 触发器同步 FTS
            CREATE TRIGGER IF NOT EXISTS experiences_ai AFTER INSERT ON experiences BEGIN
                INSERT INTO experiences_fts(rowid, id, title, content) 
                VALUES (new.rowid, new.id, new.title, new.content);
            END;",
        )
        .await?;

        Ok(Self {
            conn,
            jieba: Arc::new(Jieba::new()),
        })
    }

    /// 对中文文本进行分词，以便 FTS5 搜索
    fn tokenize(&self, text: &str) -> String {
        self.jieba.cut(text, false).join(" ")
    }

    pub async fn add_experience(&self, exp: Experience) -> Result<()> {
        let tags_json = serde_json::to_string(&exp.tags).unwrap_or_default();

        // 分词处理
        let tokenized_title = self.tokenize(&exp.title);
        let tokenized_content = self.tokenize(&exp.content);

        self.conn
            .execute(
                "INSERT INTO experiences (id, title, content, tags, created_at) VALUES (?, ?, ?, ?, ?)",
                params![exp.id, tokenized_title, tokenized_content, tags_json, exp.created_at],
            )
            .await?;
        Ok(())
    }

    pub async fn search(&self, query_str: &str, limit: usize) -> Result<Vec<Experience>> {
        // 搜索词也需要分词
        let tokenized_query = self.tokenize(query_str);

        let mut rows = self
            .conn
            .query(
                "SELECT e.id, e.title, e.content, e.tags, e.created_at 
                 FROM experiences e
                 JOIN experiences_fts f ON e.rowid = f.rowid
                 WHERE experiences_fts MATCH ?
                 ORDER BY rank 
                 LIMIT ?",
                params![tokenized_query, limit as i64],
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
