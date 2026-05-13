mod ai;
mod extractor;
mod index;
mod indexing;
mod math;
mod scanner;
mod searching;
mod vector_db;

use anyhow::{Context, Result};
use chrono::TimeZone;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(about = "Local Semantic Search Engine", long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index {
        path: Option<String>,
    },

    Search {
        query: String,

        #[arg(long)]
        ext: Option<String>,

        #[arg(long)]
        since: Option<String>,
    },

    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let proj_dirs = ProjectDirs::from("", "", "semantic-search")
        .context("Could not determine project directories")?;
    let db_path = proj_dirs.data_dir().join("vectors.db");
    let bm25_dir = proj_dirs.data_dir().join("bm25_index");

    let db = vector_db::VectorDb::open(&db_path)?;
    let current_model_id = "fastembed/Gemma-300M";

    match cli.command {
        Commands::Index { path } => {
            if let Some(stored_id) = db.get_model_id()? {
                if stored_id != current_model_id {
                    anyhow::bail!(
                        "Database was created with model '{}', but you are using '{}'. \
                        Please run 'clean' first.",
                        stored_id,
                        current_model_id
                    );
                }
            } else {
                db.set_model_id(current_model_id)?;
            }

            #[cfg(target_os = "windows")]
            let default_path = "C:\\Users";
            #[cfg(not(target_os = "windows"))]
            let default_path = "/";

            let path_to_scan = path.as_deref().unwrap_or(default_path);
            println!("Scanning '{}'...", path_to_scan);

            let files = scanner::scan_all(path_to_scan);
            println!("Found {} files", files.len());

            let mut engine = ai::AiEngine::new()?;
            let bm25 = index::IndexManager::open_or_create(&bm25_dir)?;

            let (indexed, skipped) = indexing::run(&mut engine, &db, &bm25, files)?;
            println!("Done. Indexed: {}, Skipped: {}", indexed, skipped);
            println!("Total chunks in index: {}", db.chunk_count()?);
        }

        Commands::Search { query, ext, since } => {
            if let Some(stored_id) = db.get_model_id()?.filter(|id| id != current_model_id) {
                anyhow::bail!(
                    "Search error: The index is incompatible. \
                    Base: '{}', Current model: '{}'.",
                    stored_id,
                    current_model_id
                );
            }

            let since_filter = since.and_then(|s| {
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| chrono::Utc.from_utc_datetime(&dt).timestamp())
            });

            let mut engine = ai::AiEngine::new()?;
            searching::run(
                &query,
                ext.as_deref(),
                since_filter,
                &mut engine,
                &db,
                &bm25_dir,
            )?;
        }

        Commands::Clean => {
            let data_dir = proj_dirs.data_dir();
            if data_dir.exists() {
                std::fs::remove_dir_all(data_dir).context("Failed to clear index directory")?;
            }
            println!("Index cleared.");
        }
    }

    Ok(())
}
