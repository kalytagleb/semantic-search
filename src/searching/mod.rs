use crate::ai::AiEngine;
use crate::math;
use crate::vector_db::VectorDb;
use rayon::prelude::*;

struct Hit {
    path: String,
    text: String,
    score: f32,
}

pub fn run(query: &str, engine: &mut AiEngine, db: &VectorDb) -> anyhow::Result<()> {
    let records = db.load_all_vectors()?;

    if records.is_empty() {
        println!("Index is empty. Run 'index <path>' first.");
        return Ok(());
    }

    println!("Searching {} chunks for: '{}'", records.len(), query);

    let query_vec = engine.generate_query_embedding(query)?;

    let mut hits: Vec<Hit> = records
        .par_iter()
        .map(|record| Hit {
            path: record.path.clone(),
            text: record.chunk_text.clone(),
            score: math::cosine_similarity(&query_vec, &record.vector),
        })
        .collect();

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nTop 10 results:");
    for (i, hit) in hits.iter().take(10).enumerate() {
        println!("\n{}. Score: {:.4}", i + 1, hit.score);
        println!("   Path: {}", hit.path);
        println!("   Text: {}", truncate_preview(&hit.text, 120));
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
