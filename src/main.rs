use std::env;

use anyhow::Context;
use directories::ProjectDirs;

mod ai;
mod extractor;
mod index;
mod indexing;
mod math;
mod scanner;
mod searching;
mod vector_db;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let data = data_dir()?;
    let db_path = data.join("vectors.db");
    let bm25_dir = data.join("bm25-index");

    let db = vector_db::VectorDb::open(&db_path)?;

    match args.get(1).map(|s| s.as_str()) {
        Some("index") => {
            let scan_path = args.get(2).map(|s| s.as_str()).unwrap_or("/");
            let mut engine = ai::AiEngine::new()?;

            if let Some(stored_model) = db.get_model_id()?
                && stored_model != ai::MODEL_ID
            {
                println!(
                    "Model changed ('{}' → '{}'). Clearing old index...",
                    stored_model,
                    ai::MODEL_ID
                );
                db.clear_chunks()?;
                if bm25_dir.exists() {
                    std::fs::remove_dir_all(&bm25_dir)?;
                }
            }

            indexing::run(scan_path, &db, &mut engine, &bm25_dir)?;
            db.set_model_id(ai::MODEL_ID)?;
        }

        Some("search") => {
            let query = args
                .get(2)
                .context("Usage: semantic-search search <query>")?;
            let mut engine = ai::AiEngine::new()?;
            searching::run(query, &mut engine, &db, &bm25_dir)?;
        }

        Some("clean") => {
            let data = data_dir()?;
            if data.exists() {
                std::fs::remove_dir_all(data)?;
                println!("Index cleared.");
            } else {
                println!("No index found to clear.");
            }
        }

        _ => {
            println!("Local Semantic Search Engine");
            println!();
            println!("Usage:");
            println!("  semantic-search index [path]    index files (default: /)");
            println!("  semantic-search search <query>  search the index");
            println!("  semantic-search clean           clear the index");
            println!();
            println!("Examples:");
            println!("  semantic-search index ~/Documents");
            println!("  semantic-search index /");
            println!("  semantic-search search \"snake game in python\"");
            println!("  semantic-search clean");
            println!();
            println!("Index stored at: {}", db_path.display());
        }
    }

    Ok(())
}

fn data_dir() -> anyhow::Result<std::path::PathBuf> {
    ProjectDirs::from("", "", "semantic-search")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .context("Could not determine user data directory")
}
