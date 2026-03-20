pub mod schema;

use std::path::Path;

use std::fs::create_dir_all;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexWriter, doc};

use crate::extractor;
use crate::index::schema::IndexSchema;

// Search - is that, what user will see.
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub score: f32, // relevance
}

pub struct IndexManager {
    index: Index,
    schema: IndexSchema,
    writer: IndexWriter,
}

impl IndexManager {
    pub fn new(index_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        create_dir_all(index_dir)?;

        let schema = IndexSchema::new();
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_dir)?,
            schema.schema.clone(),
        )?;

        let writer = index.writer(50_000_000)?;
        Ok(IndexManager {
            index,
            schema,
            writer,
        })
    }

    pub fn add_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let extracted = extractor::extract(path).map_err(|e| e.to_string())?;

        let path_str = path.to_string_lossy().to_string();
        let title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for chunk in extracted.chunks {
            self.writer.add_document(doc!(
                self.schema.path => path_str.clone(),
                self.schema.title => title.clone(),
                self.schema.body => chunk
            ))?;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.commit()?;
        Ok(())
    }

    pub fn search(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommit)
            .try_into()?;

        let searcher = reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.schema.title, self.schema.body]);
        let query = query_parser.parse_query(query_text)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;

            let path = retrieved_doc
                .get_first(self.schema.path)
                .and_then(|v| match v {
                    tantivy::schema::Value::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("")
                .to_string();

            let title = retrieved_doc
                .get_first(self.schema.title)
                .and_then(|v| match v {
                    tantivy::schema::Value::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("")
                .to_string();

            results.push(SearchResult { path, title, score });
        }

        let mut seen = std::collections::HashMap::new();
        for result in results {
            seen.entry(result.path.clone()).or_insert(result);
        }

        let mut deduped: Vec<SearchResult> = seen.into_values().collect();
        deduped.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(deduped)
    }
}
