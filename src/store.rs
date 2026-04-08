use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::{Collector, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::{
    FAST, IndexRecordOption, STORED, STRING, Schema, TEXT, TextAttributeInfo, TextOptions,
};
use tantivy::{DocAddress, Index, IndexWriter, Order, ReloadPolicy, doc};
use tantivy_jieba::JiebaTokenizer;

#[derive(Serialize, Deserialize, Debug)]
pub struct Experience {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: i64,
}

pub struct ExperienceStore {
    index: Index,
    reader: tantivy::IndexReader,
    schema: Schema,
}

impl ExperienceStore {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // id is a unique string
        let id = schema_builder.add_text_field("id", STRING | STORED);

        // title and content use Chinese tokenizer
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextAttributeInfo::default()
                    .set_tokenizer("jieba")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let title = schema_builder.add_text_field("title", text_options.clone());
        let content = schema_builder.add_text_field("content", text_options);

        // tags is a list of strings
        let tags = schema_builder.add_text_field("tags", STRING | STORED);

        // created_at is a timestamp
        let created_at = schema_builder.add_i64_field("created_at", STORED | FAST);

        let schema = schema_builder.build();

        let index_path = path.as_ref();
        if !index_path.exists() {
            std::fs::create_dir_all(index_path)?;
        }

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_path)?,
            schema.clone(),
        )?;

        // Register Jieba tokenizer
        index
            .tokenizers()
            .register("jieba", JiebaTokenizer::default());

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            schema,
        })
    }

    pub fn add_experience(&self, exp: Experience) -> Result<()> {
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;

        let id_field = self.schema.get_field("id").unwrap();
        let title_field = self.schema.get_field("title").unwrap();
        let content_field = self.schema.get_field("content").unwrap();
        let tags_field = self.schema.get_field("tags").unwrap();
        let created_at_field = self.schema.get_field("created_at").unwrap();

        let mut doc = doc!(
            id_field => exp.id,
            title_field => exp.title,
            content_field => exp.content,
            created_at_field => exp.created_at,
        );

        for tag in exp.tags {
            doc.add_text(tags_field, tag);
        }

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<Experience>> {
        let searcher = self.reader.searcher();
        let id_field = self.schema.get_field("id").unwrap();
        let title_field = self.schema.get_field("title").unwrap();
        let content_field = self.schema.get_field("content").unwrap();
        let tags_field = self.schema.get_field("tags").unwrap();
        let created_at_field = self.schema.get_field("created_at").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![title_field, content_field]);
        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| Error::Init(format!("Query parse error: {}", e)))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;

            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let title = retrieved_doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let content = retrieved_doc
                .get_first(content_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let created_at = retrieved_doc
                .get_first(created_at_field)
                .and_then(|v| v.as_i64())
                .unwrap_or_default();

            let tags = retrieved_doc
                .get_all(tags_field)
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            results.push(Experience {
                id,
                title,
                content,
                tags,
                created_at,
            });
        }

        Ok(results)
    }
}
