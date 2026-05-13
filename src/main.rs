mod ai;
mod extractor;
mod index;
mod indexing;
mod math;
mod scanner;
mod searching;
mod tui;
mod vector_db;

use anyhow::Context;
use chrono::TimeZone;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::sync::mpsc;
use std::thread;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(about = "Local Semantic Search Engine", long_about=None)]
struct Cli {
    /// Directory to scan (used when no subcommand is given)
    path: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
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

    Gui {
        path: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let proj_dirs = ProjectDirs::from("", "", "semantic-search")
        .context("Could not determine project directories")?;
    let db_path = proj_dirs.data_dir().join("vectors.db");
    let bm25_dir = proj_dirs.data_dir().join("bm25_index");

    let db = vector_db::VectorDb::open(&db_path)?;
    let current_model_id = "fastembed/Gemma-300M";

    match cli.command.unwrap_or(Commands::Gui { path: cli.path }) {
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

            let (indexed, skipped) = indexing::run(&mut engine, &db, &bm25, files, |_, _, _| {})?;
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

        Commands::Gui { path } => {
            let (tx_to_tui, rx_for_tui) = mpsc::channel::<tui::events::BackendEvent>();
            let (tx_to_back, rx_for_back) = mpsc::channel::<tui::events::FrontendEvent>();

            #[cfg(target_os = "windows")]
            let default_path = "C:\\Users";
            #[cfg(not(target_os = "windows"))]
            let default_path = "/";
            let path_to_scan = path.as_deref().unwrap_or(default_path).to_string();

            let db_path_thread = db_path.clone();
            let bm25_dir_thread = bm25_dir.clone();

            thread::spawn(move || {
                if let Err(e) = run_backend(
                    tx_to_tui,
                    rx_for_back,
                    db_path_thread,
                    bm25_dir_thread,
                    path_to_scan,
                ) {
                    eprintln!("Backend error: {}", e);
                }
            });

            let app = tui::TuiApp::new(tx_to_back, rx_for_tui);
            app.run().context("TUI error")?;
        }
    }

    Ok(())
}

fn run_backend(
    tx: mpsc::Sender<tui::events::BackendEvent>,
    rx: mpsc::Receiver<tui::events::FrontendEvent>,
    db_path: std::path::PathBuf,
    bm25_dir: std::path::PathBuf,
    path_to_scan: String,
) -> anyhow::Result<()> {
    use tui::events::{BackendEvent, FrontendEvent};

    let db = vector_db::VectorDb::open(&db_path)?;
    let bm25 = index::IndexManager::open_or_create(&bm25_dir)?;
    let _ = tx.send(BackendEvent::Progress(
        0.0,
        "Loading AI model...".to_string(),
    ));
    let mut engine = ai::AiEngine::new()?;

    let files = scanner::scan_all(&path_to_scan);
    let total = files.len();
    let _ = tx.send(BackendEvent::Progress(
        0.0,
        format!("Found {} files to index", total),
    ));

    let tx_progress = tx.clone();
    indexing::run(&mut engine, &db, &bm25, files, move |done, total, path| {
        if total > 0 {
            let _ = tx_progress.send(BackendEvent::Progress(
                done as f64 / total as f64,
                path.to_string(),
            ));
        }
    })?;

    let _ = tx.send(BackendEvent::IndexingFinished);

    for event in rx {
        match event {
            FrontendEvent::Search(query) => {
                match searching::search(&query, None, None, &mut engine, &db, &bm25_dir) {
                    Ok(results) => {
                        let _ = tx.send(BackendEvent::SearchResults(results));
                    }
                    Err(e) => {
                        let _ = tx.send(BackendEvent::Error(e.to_string()));
                    }
                }
            }
            FrontendEvent::Quit => break,
        }
    }

    Ok(())
}
