use std::env;

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
    }
}

fn run() -> anyhow::Result<()> {
    let mut engine = ai::AiEngine::new()?;
    println!("AI Engine loaded successfully");

    indexing::run("./my_test_folder")?;

    if let Some(query) = env::args().nth(1) {
        searching::run(&query, &mut engine)?;
    }

    Ok(())
}
