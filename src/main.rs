use std::path::Path;

mod extractor;
mod index;
mod scanner;
mod ai;

fn main() {
    let mut engine = ai::AiEngine::new().expect("Failed to load model");

    let text = "Hello world";
    let vector = engine.generate_embedding(text).unwrap();

    println!("vector for '{}': {:?}", text, &vector[..5]);
    println!("total dimensions: {}", vector.len());
}

// fn main() {
//     let index_dir = Path::new(".search-index");

//     let mut manager = match index::IndexManager::new(index_dir) {
//         Ok(m) => m,
//         Err(e) => {
//             println!("Error: {}", e);
//             return;
//         }
//     };

//     let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

//     println!("Scanning: {}", root);

//     let files = scanner::scan_all(&root);

//     println!("There are: {} files", files.len());

//     let mut indexed = 0;
//     for file in &files {
//         match manager.add_file(file) {
//             Ok(()) => indexed += 1,
//             Err(e) => println!("[Skip] {}: {}", file.display(), e),
//         }
//     }

//     if let Err(e) = manager.commit() {
//         println!("Commit error: {}", e);
//         return;
//     }

//     println!("Indexed: {} files\n", indexed);

//     let query = std::env::args().nth(2).unwrap_or_else(|| "crc".to_string());

//     println!("Search: '{}'\n", query);

//     match manager.search(&query, 20) {
//         Ok(results) => {
//             if results.is_empty() {
//                 println!("Nothing found.");
//             } else {
//                 for r in results {
//                     println!("[{:.2}] {} - {}", r.score, r.title, r.path);
//                 }
//             }
//         }
//         Err(e) => println!("Error of search: {}", e),
//     }
// }
