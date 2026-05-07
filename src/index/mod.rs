use crate::index::schema::IndexSchema;
use anyhow::Result;
use std::path::Path;
use tantivy::query::QueryParser;
use tantivy::Index;

pub mod schema;

pub struct IndexManager {
    pub schema: IndexSchema,
    pub index: Index,
}

impl IndexManager {
    pub fn open_or_create(index_path: &Path) -> Result<Self> {
        let schema = IndexSchema::new();
        if !index_path.exists() {
            std::fs::create_dir_all(index_path)?;
        }
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_path)?,
            schema.schema.clone(),
        )?;

        Ok(IndexManager { schema, index })
    }

    pub fn add_chunks(
        &self,
        path: &str,
        extension: &str,
        _title: &str,
        chunks: &[String],
    ) -> Result<()> {
        let mut writer = self.index.writer(50_000_000)?;
        let schema = &self.schema;

        for (i, chunk) in chunks.iter().enumerate() {
            let mut doc = tantivy::Document::new();
            doc.add_text(schema.path, path);
            doc.add_text(schema.extension, extension);
            doc.add_u64(schema.chunk_index, i as u64);
            doc.add_text(schema.text, chunk);
            writer.add_document(doc)?;
        }

        writer.commit()?;
        Ok(())
    }

    pub fn delete_file(&self, path: &str) {
        let mut writer = self.index.writer(50_000_000).unwrap();
        let term = tantivy::Term::from_field_text(self.schema.path, path);
        writer.delete_term(term);
        writer.commit().unwrap();
    }
}

pub fn search_bm25(
    index_path: &Path,
    query_str: &str,
    extension_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<crate::vector_db::SearchResult>> {
    let schema_def = IndexSchema::new();
    let index = Index::open_or_create(
        tantivy::directory::MmapDirectory::open(index_path)?,
        schema_def.schema.clone(),
    )?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![schema_def.text]);
    let text_query = query_parser.parse_query(query_str)?;

    let final_query: Box<dyn tantivy::query::Query> = if let Some(ext) = extension_filter {
        let ext_term = tantivy::Term::from_field_text(schema_def.extension, ext);
        let ext_query = tantivy::query::TermQuery::new(
            ext_term,
            tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
        );
        Box::new(tantivy::query::BooleanQuery::new(vec![
            (tantivy::query::Occur::Must, Box::new(text_query)),
            (tantivy::query::Occur::Must, Box::new(ext_query)),
        ]))
    } else {
        text_query
    };

    let top_docs =
        searcher.search(&final_query, &tantivy::collector::TopDocs::with_limit(limit))?;

    let mut results = Vec::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let path = retrieved_doc
            .get_first(schema_def.path)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();
        results.push(crate::vector_db::SearchResult {
            path,
            score: 1.0,
            text: "".to_string(),
        });
    }

    Ok(results)
}
