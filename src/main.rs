use std::env;

mod extractor;
mod index;
mod scanner;
mod ai;
mod vector_db;
mod math;
mod indexing;
mod searching;

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
