use crate::ai::AiEngine;
use crate::index::IndexManager;
use crate::vector_db::{VectorDb, VectorRecord};
use crate::{extractor, scanner};
use sha2::{Digest, Sha256};
use std::path::Path;

const BATCH_SIZE: usize = 16;

pub fn run(
    folder_path: &str,
    db: &VectorDb,
    engine: &mut AiEngine,
    bm25_dir: &Path,
) -> anyhow::Result<()> {
    println!("Scanning '{}'...", folder_path);
    let files = scanner::scan_all(folder_path);
    println!("Found {} files", files.len());

    if files.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    let mut bm25 = IndexManager::new(bm25_dir)?;
    let mut indexed = 0usize;
    let mut skipped = 0usize;

    for file_path in &files {
        let content = match extractor::text::read(file_path) {
            Ok(c) => c,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let hash = hash_content(&content);
        let path_str = file_path.to_string_lossy().to_string();

        if !db.file_needs_reindex(&path_str, &hash)? {
            skipped += 1;
            continue;
        }

        let file_type = match scanner::filters::get_file_type(file_path) {
            Some(ft) => ft,
            None => {
                skipped += 1;
                continue;
            }
        };

        let raw_chunks = extractor::chunker::split(&content, &file_type);
        if raw_chunks.is_empty() {
            skipped += 1;
            continue;
        }

        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Remove stale data from both stores
        db.delete_file_chunks(&path_str)?;
        bm25.delete_file(&path_str);

        // Index into BM25 (original text chunks, no prefix)
        bm25.add_chunks(&path_str, &file_name, &raw_chunks)?;

        // Index into vector store (with passage prefix for better semantic quality)
        embed_and_store(engine, db, &path_str, &hash, &file_name, &raw_chunks)?;

        indexed += 1;
    }

    bm25.commit()?;

    println!(
        "Done. Indexed: {}, Skipped (unchanged/unreadable): {}",
        indexed, skipped
    );
    println!("Total chunks in index: {}", db.chunk_count()?);

    Ok(())
}

fn embed_and_store(
    engine: &mut AiEngine,
    db: &VectorDb,
    path: &str,
    hash: &str,
    file_name: &str,
    raw_chunks: &[String],
) -> anyhow::Result<()> {
    for (batch_i, batch) in raw_chunks.chunks(BATCH_SIZE).enumerate() {
        // Embed with passage prefix + filename for better semantic quality
        let to_embed: Vec<String> = batch
            .iter()
            .map(|c| format!("passage: File: {}.\n{}", file_name, c))
            .collect();

        let texts: Vec<&str> = to_embed.iter().map(|s| s.as_str()).collect();
        let vectors = engine.generate_embeddings_batch(texts)?;

        for (i, (raw_chunk, vector)) in batch.iter().zip(vectors.into_iter()).enumerate() {
            db.insert_chunk(&VectorRecord {
                path: path.to_string(),
                content_hash: hash.to_string(),
                chunk_index: batch_i * BATCH_SIZE + i,
                chunk_text: raw_chunk.clone(),
                vector,
            })?;
        }
    }
    Ok(())
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
