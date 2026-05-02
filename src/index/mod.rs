pub mod schema;

use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexWriter, doc};

use crate::index::schema::IndexSchema;

pub struct SearchResult {
    pub path: String,
    #[allow(dead_code)]
    pub title: String,
    pub score: f32,
}

pub struct IndexManager {
    schema: IndexSchema,
    writer: IndexWriter,
}

impl IndexManager {
    pub fn new(index_dir: &Path) -> anyhow::Result<Self> {
        create_dir_all(index_dir)?;

        let schema = IndexSchema::new();
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_dir)?,
            schema.schema.clone(),
        )?;

        let writer = index.writer(50_000_000)?;
        Ok(IndexManager { schema, writer })
    }

    pub fn add_chunks(
        &mut self,
        path: &str,
        title: &str,
        chunks: &[String],
    ) -> anyhow::Result<()> {
        for chunk in chunks {
            self.writer.add_document(doc!(
                self.schema.path  => path,
                self.schema.title => title,
                self.schema.body  => chunk.as_str()
            ))?;
        }
        Ok(())
    }

    pub fn delete_file(&mut self, path: &str) {
        let term = tantivy::Term::from_field_text(self.schema.path, path);
        self.writer.delete_term(term);
    }

    pub fn commit(&mut self) -> anyhow::Result<()> {
        self.writer.commit()?;
        Ok(())
    }
}

/// Opens the BM25 index read-only (no writer lock) and returns ranked results.
pub fn search_bm25(
    index_dir: &Path,
    query_text: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchResult>> {
    if !index_dir.exists() {
        return Ok(vec![]);
    }

    let schema_def = IndexSchema::new();
    let index = Index::open_or_create(
        tantivy::directory::MmapDirectory::open(index_dir)?,
        schema_def.schema.clone(),
    )?;

    let reader = index
        .reader_builder()
        .reload_policy(tantivy::ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();

    let query_parser =
        QueryParser::for_index(&index, vec![schema_def.title, schema_def.body]);

    let query = match query_parser.parse_query(query_text) {
        Ok(q) => q,
        Err(_) => return Ok(vec![]),
    };

    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit * 5))?;

    // Deduplicate by path, keep best score
    let mut seen: HashMap<String, (String, f32)> = HashMap::new();
    for (score, doc_addr) in top_docs {
        let retrieved = searcher.doc(doc_addr)?;

        let path = retrieved
            .get_first(schema_def.path)
            .and_then(|v| match v {
                tantivy::schema::Value::Str(s) => Some(s.as_str().to_string()),
                _ => None,
            })
            .unwrap_or_default();

        let title = retrieved
            .get_first(schema_def.title)
            .and_then(|v| match v {
                tantivy::schema::Value::Str(s) => Some(s.as_str().to_string()),
                _ => None,
            })
            .unwrap_or_default();

        if !path.is_empty() {
            seen.entry(path).or_insert((title, score));
        }
    }

    let mut results: Vec<SearchResult> = seen
        .into_iter()
        .map(|(path, (title, score))| SearchResult { path, title, score })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    Ok(results)
}
