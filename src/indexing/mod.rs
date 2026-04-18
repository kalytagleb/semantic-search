use crate::ai::AiEngine;
use crate::vector_db::VectorRecord;
use crate::{extractor, scanner};
use std::path::{Path, PathBuf};

const BATCH_SIZE: usize = 16;
const DB_PATH: &str = "vectors_db.bin";

pub fn run(folder_path: &str) -> anyhow::Result<Vec<VectorRecord>> {
    println!("scanning files");
    let files = scanner::scan_all(folder_path);
    println!("Found {} files", files.len());

    if files.is_empty() {
        println!("No files found. Exiting.");
        return Ok(Vec::new());
    }

    println!("\ninitializing AI Engine");
    let mut engine = AiEngine::new()?;
    println!("AI Engine loaded successfully");

    println!("processing files");
    let all_records = process_files_in_batches(&files, &mut engine)?;

    println!("\nSaving to database");
    println!("Total records to save: {}", all_records.len());
    crate::vector_db::save_db(&all_records, Path::new(DB_PATH))?;
    println!("Saved to {}", DB_PATH);

    Ok(all_records)
}

fn process_files_in_batches(
    files: &[PathBuf],
    engine: &mut AiEngine,
) -> anyhow::Result<Vec<VectorRecord>> {
    let mut all_records: Vec<VectorRecord> = Vec::new();

    for file_path in files {
        println!("Processing: {}", file_path.display());

        let content = match extractor::text::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                println!("  Error reading file: {}", e);
                continue;
            }
        };
        println!("  Read {} bytes", content.len());

        let file_type = match scanner::filters::get_file_type(file_path) {
            Some(ft) => ft,
            None => {
                println!("  Error: unsupported file type");
                continue;
            }
        };

        let chunks = extractor::chunker::split(&content, &file_type);
        println!("  Split into {} chunks", chunks.len());

        process_chunks_for_file(&mut all_records, engine, file_path, chunks)?;
    }

    Ok(all_records)
}

fn process_chunks_for_file(
    all_records: &mut Vec<VectorRecord>,
    engine: &mut AiEngine,
    file_path: &Path,
    chunks: Vec<String>,
) -> anyhow::Result<()> {
    let mut batch: Vec<&str> = Vec::new();
    let mut batch_metadata: Vec<(String, String)> = Vec::new();

    for chunk in &chunks {
        batch.push(chunk);
        batch_metadata.push((file_path.to_string_lossy().to_string(), chunk.clone()));

        if batch.len() == BATCH_SIZE {
            process_batch(all_records, engine, batch, batch_metadata)?;
            batch = Vec::new();
            batch_metadata = Vec::new();
        }
    }

    if !batch.is_empty() {
        let batch_size = batch.len();
        process_batch(all_records, engine, batch, batch_metadata)?;
        println!("    Processed final batch of {} chunks", batch_size);
    }

    Ok(())
}

fn process_batch(
    all_records: &mut Vec<VectorRecord>,
    engine: &mut AiEngine,
    batch: Vec<&str>,
    batch_metadata: Vec<(String, String)>,
) -> anyhow::Result<()> {
    let vectors = engine.generate_embeddings_batch(batch)?;

    let mut mutable_metadata = batch_metadata;
    for (vector, (path, text)) in vectors.into_iter().zip(mutable_metadata.drain(..)) {
        all_records.push(VectorRecord {
            path,
            chunk_text: text,
            vector,
        });
    }
    println!("    Processed batch of {} chunks", BATCH_SIZE);

    Ok(())
}
