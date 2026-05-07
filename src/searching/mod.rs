use crate::ai::AiEngine;
use crate::index::search_bm25;
use crate::math;
use crate::vector_db::VectorDb;
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
    let records = db.load_all_vectors()?;

    // Apply filters early
    let records: Vec<_> = records
        .into_iter()
        .filter(|r| {
            let ext_match = extension_filter.map_or(true, |ext| r.extension == ext);
            let since_match = since_filter.map_or(true, |since| r.mtime >= since);
            ext_match && since_match
        })
        .collect();

    if records.is_empty() {
        println!(
            "Index is empty (or no files match the specified filters). Run 'index <path>' first."
        );
        return Ok(());
    }

    println!("Searching {} chunks for: '{}'", records.len(), query);

    // --- Vector leg ---
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

    // Deduplicate vector results by path (keep best chunk per file)
    let mut vector_by_path: Vec<(String, f32, String)> = Vec::new();
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (path, score, text) in vector_hits {
        if seen_paths.insert(path.clone()) {
            vector_by_path.push((path, score, text));
        }
    }

    // --- BM25 leg ---
    let bm25_results = search_bm25(bm25_dir, query, extension_filter, 100)?;

    // --- Reciprocal Rank Fusion (k=60) ---
    let mut rrf_scores: HashMap<String, f32> = HashMap::new();

    for (rank, (path, _, _)) in vector_by_path.iter().enumerate() {
        *rrf_scores.entry(path.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32);
    }
    for (rank, result) in bm25_results.iter().enumerate() {
        *rrf_scores.entry(result.path.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32);
    }

    // Sort by RRF score
    let mut ranked: Vec<(String, f32)> = rrf_scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nTop 10 results:");
    for (i, (path, rrf)) in ranked.iter().take(10).enumerate() {
        let preview = vector_by_path
            .iter()
            .find(|(p, _, _)| p == path)
            .map(|(_, _, t)| t.as_str())
            .unwrap_or("");

        println!("\n{}. Score: {:.4}", i + 1, rrf);
        println!("   Path: {}", path);
        if !preview.is_empty() {
            println!("   Text: {}", truncate_preview(preview, 120));
        }
    }

    Ok(())
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{}...", truncated)
}
