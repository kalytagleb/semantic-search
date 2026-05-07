mod ai;
mod extractor;
mod index;
mod indexing;
mod math;
mod scanner;
mod searching;
mod vector_db;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let proj_dirs = ProjectDirs::from("", "", "semantic-search")
        .context("Could not determine project directories")?;
    let db_path = proj_dirs.data_dir().join("vectors.db");
    let bm25_dir = proj_dirs.data_dir().join("bm25_index");

    let db = vector_db::VectorDb::open(&db_path)?;

    match args.get(1).map(|s| s.as_str()) {
        Some("index") => {
            // Default path logic
            #[cfg(target_os = "windows")]
            let default_path = "C:\\Users";
            #[cfg(not(target_os = "windows"))]
            let default_path = "/";

            let path = args.get(2).map(|s| s.as_str()).unwrap_or(default_path);
            println!("Scanning '{}'...", path);

            let files = scanner::scan_all(path);
            println!("Found {} files", files.len());

            let mut engine = ai::AiEngine::new()?;
            let bm25 = index::IndexManager::open_or_create(&bm25_dir)?;

            let (indexed, skipped) = indexing::run(&mut engine, &db, &bm25, files)?;
            println!(
                "Done. Indexed: {}, Skipped (unchanged/unreadable): {}",
                indexed, skipped
            );
            println!("Total chunks in index: {}", db.chunk_count()?);
        }

        Some("search") => {
            let query = args
                .get(2)
                .context("Usage: semantic-search search <query>")?;

            let extension_filter = args
                .windows(2)
                .find(|w| w[0] == "--ext")
                .map(|w| w[1].as_str());

            let since_filter = args
                .windows(2)
                .find(|w| w[0] == "--since")
                .and_then(|w| chrono::NaiveDate::parse_from_str(&w[1], "%Y-%m-%d").ok())
                .and_then(|d: chrono::NaiveDate| d.and_hms_opt(0, 0, 0))
                .map(|dt| chrono::TimeZone::from_utc_datetime(&chrono::Utc, &dt).timestamp());

            let mut engine = ai::AiEngine::new()?;
            searching::run(
                query,
                extension_filter,
                since_filter,
                &mut engine,
                &db,
                &bm25_dir,
            )?;
        }

        Some("clean") => {
            let data_dir = proj_dirs.data_dir();
            if data_dir.exists() {
                std::fs::remove_dir_all(data_dir).context("Failed to clear index directory")?;
            }
            println!("Index cleared.");
        }

        _ => {
            println!("Local Semantic Search Engine");
            println!();
            println!("Usage:");
            #[cfg(target_os = "windows")]
            println!("  semantic-search index [path]    index files (default: C:\\Users)");
            #[cfg(not(target_os = "windows"))]
            println!("  semantic-search index [path]    index files (default: /)");

            println!("  semantic-search search <query>  search the index");
            println!("  semantic-search clean           clear the index");
            println!();
            println!("Options for search:");
            println!("  --ext <extension>               filter by extension (e.g., --ext rs)");
            println!(
                "  --since <YYYY-MM-DD>            filter by modification date (e.g., 2024-01-01)"
            );
            println!();
            println!("Examples:");
            println!("  semantic-search index ~/Documents");
            #[cfg(target_os = "windows")]
            println!("  semantic-search index C:\\Users");
            #[cfg(not(target_os = "windows"))]
            println!("  semantic-search index /");
            println!("  semantic-search search \"snake game in python\"");
            println!("  semantic-search search \"bug report\" --ext rs --since 2024-05-01");
            println!("  semantic-search clean");
            println!();
            println!("Index stored at: {}", db_path.display());
        }
    }

    Ok(())
}
