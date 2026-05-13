use crate::ai::AiEngine;
use crate::extractor;
use crate::extractor::chunker;
use crate::index::IndexManager;
use crate::scanner::filters::get_file_type;
use crate::vector_db::{VectorDb, VectorRecord};
use anyhow::Result;
use sha2::{Digest, Sha256};

const BATCH_SIZE: usize = 16;

pub fn run(
    engine: &mut AiEngine,
    db: &VectorDb,
    bm25: &IndexManager,
    files: Vec<std::path::PathBuf>,
    progress_cb: impl Fn(usize, usize, &str),
) -> Result<(usize, usize)> {
    let mut indexed = 0;
    let mut skipped = 0;
    let total = files.len();

    for (i, file_path) in files.into_iter().enumerate() {
        let path_str = file_path.to_string_lossy().to_string();
        progress_cb(i, total, &path_str);
        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = file_path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mtime = file_path
            .metadata()
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let content = match extractor::extract_text(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to extract text from {}: {}", path_str, e);
                skipped += 1;
                continue;
            }
        };

        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));

        if !db.file_needs_reindex(&path_str, &hash)? {
            skipped += 1;
            continue;
        }

        let file_type = match get_file_type(&file_path) {
            Some(ft) => ft,
            None => {
                skipped += 1;
                continue;
            }
        };

        let raw_chunks = chunker::split(&content, &file_type);

        // Delete old chunks before re-indexing
        db.delete_file_chunks(&path_str)?;
        bm25.delete_file(&path_str);

        // Index into BM25
        bm25.add_chunks(&path_str, &extension, mtime, &file_name, &raw_chunks)?;

        // Index into vector store
        embed_and_store(
            engine,
            db,
            &path_str,
            &extension,
            mtime,
            &hash,
            &file_name,
            &raw_chunks,
        )?;

        indexed += 1;
    }

    Ok((indexed, skipped))
}

#[allow(clippy::too_many_arguments)]
fn embed_and_store(
    engine: &mut AiEngine,
    db: &VectorDb,
    path: &str,
    extension: &str,
    mtime: i64,
    hash: &str,
    file_name: &str,
    raw_chunks: &[String],
) -> anyhow::Result<()> {
    for (batch_i, batch) in raw_chunks.chunks(BATCH_SIZE).enumerate() {
        let to_embed: Vec<String> = batch
            .iter()
            .map(|c| format!("passage: File: {}.\n{}", file_name, c))
            .collect();

        let texts: Vec<&str> = to_embed.iter().map(|s| s.as_str()).collect();
        let vectors = engine.generate_embeddings_batch(texts)?;

        for (i, (raw_chunk, vector)) in batch.iter().zip(vectors).enumerate() {
            db.insert_chunk(&VectorRecord {
                path: path.to_string(),
                extension: extension.to_string(),
                mtime,
                content_hash: hash.to_string(),
                chunk_index: batch_i * BATCH_SIZE + i,
                chunk_text: raw_chunk.clone(),
                vector,
            })?;
        }
    }
    Ok(())
}
