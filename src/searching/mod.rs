use crate::ai::AiEngine;
use crate::math;
use rayon::prelude::*;
use std::path::Path;

struct Hit {
    path: String,
    text: String,
    score: f32,
}

pub fn run(query: &str, engine: &mut AiEngine) -> anyhow::Result<()> {
    let db_path = Path::new("vectors_db.bin");

    println!("\nLoading from database");
    let loaded_records = crate::vector_db::load_db(db_path)?;

    for (i, record) in loaded_records.iter().enumerate().take(5) {
        println!(
            "Record {}: path='{}', chunk_len={}, vector_dims={}",
            i + 1,
            record.path,
            record.chunk_text.len(),
            record.vector.len()
        );
    }

    println!("\nSearching for: '{}'", query);

    let query_embedding = engine.generate_embedding(query)?;

    let hits: Vec<Hit> = loaded_records
        .par_iter()
        .map(|record| Hit {
            path: record.path.clone(),
            text: record.chunk_text.clone(),
            score: math::cosine_similarity(&query_embedding, &record.vector),
        })
        .collect();

    let mut sorted_hits = hits;
    sorted_hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("\nTop 5 results:");
    for (i, hit) in sorted_hits.iter().take(5).enumerate() {
        println!("\n{}. Score: {:.4}", i + 1, hit.score);
        println!("   Path: {}", hit.path);

        let preview = format_text_preview(&hit.text);
        println!("   Text: {}", preview);
    }

    Ok(())
}

fn format_text_preview(text: &str) -> String {
    if text.len() > 100 {
        let mut end = 100;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    } else {
        text.to_string()
    }
}
