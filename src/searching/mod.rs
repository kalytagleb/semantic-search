use crate::ai::AiEngine;
use crate::index::search_bm25;
use crate::math;
use crate::vector_db::{SearchResult, VectorDb};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub fn run(
    query: &str,
    extension_filter: Option<&str>,
    since_filter: Option<i64>,
    engine: &mut AiEngine,
    db: &VectorDb,
    bm25_dir: &Path,
) -> anyhow::Result<()> {
    let results = search(query, extension_filter, since_filter, engine, db, bm25_dir)?;

    if results.is_empty() {
        println!(
            "Index is empty (or no files match the specified filters). Run 'index <path>' first."
        );
        return Ok(());
    }

    println!("\nTop {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("\n{}. Score: {:.4}", i + 1, result.score);
        println!("   Path: {}", result.path);
        if !result.text.is_empty() {
            println!("   Text: {}", truncate_preview(&result.text, 120));
        }
    }

    Ok(())
}

pub fn search(
    query: &str,
    extension_filter: Option<&str>,
    since_filter: Option<i64>,
    engine: &mut AiEngine,
    db: &VectorDb,
    bm25_dir: &Path,
) -> anyhow::Result<Vec<SearchResult>> {
    let records = db.load_all_vectors()?;

    let records: Vec<_> = records
        .into_iter()
        .filter(|r| {
            let ext_match = extension_filter.is_none_or(|ext| r.extension == ext);
            let since_match = since_filter.is_none_or(|since| r.mtime >= since);
            ext_match && since_match
        })
        .collect();

    if records.is_empty() {
        return Ok(Vec::new());
    }

    let query_vec = engine.generate_query_embedding(query)?;

    let mut vector_hits: Vec<(String, f32, String)> = records
        .par_iter()
        .map(|r| {
            (
                r.path.clone(),
                math::cosine_similarity(&query_vec, &r.vector),
                r.chunk_text.clone(),
            )
        })
        .collect();

    vector_hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut vector_by_path: Vec<(String, f32, String)> = Vec::new();
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (path, score, text) in vector_hits {
        if seen_paths.insert(path.clone()) {
            vector_by_path.push((path, score, text));
        }
    }

    let bm25_results = search_bm25(bm25_dir, query, extension_filter, since_filter, 100)?;

    let mut rrf_scores: HashMap<String, f32> = HashMap::new();
    for (rank, (path, _, _)) in vector_by_path.iter().enumerate() {
        *rrf_scores.entry(path.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32);
    }
    for (rank, result) in bm25_results.iter().enumerate() {
        *rrf_scores.entry(result.path.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32);
    }

    let mut ranked: Vec<(String, f32)> = rrf_scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let results = ranked
        .into_iter()
        .take(10)
        .map(|(path, score)| {
            let text = vector_by_path
                .iter()
                .find(|(p, _, _)| p == &path)
                .map(|(_, _, t)| t.clone())
                .unwrap_or_default();
            SearchResult { path, score, text }
        })
        .collect();

    Ok(results)
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{}...", truncated)
}
