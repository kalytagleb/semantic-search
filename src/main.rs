use std::{path::Path, vec};

use crate::vector_db::VectorRecord;

mod extractor;
mod index;
mod scanner;
mod ai;
mod vector_db;

fn main() {
    let records = vec![
        vector_db::VectorRecord {
            path: "1".to_string(),
            chunk_text: "2".to_string(),
            vector: vec![1., 2., 3.],
        },
        vector_db::VectorRecord {
            path: "3".to_string(),
            chunk_text: "4".to_string(),
            vector: vec![4., 5., 6.],
        },
    ];
    vector_db::save_db(&records, ".");

    let vectors_test = vector_db::load_db(".");
    for myvec in vectors_test {
        println!("{:?}", myvec.chunk_text);
    }


    // let mut engine = ai::AiEngine::new().expect("Failed to load model");

    // let texts = vec![
    //     "Java is a programming language",
    //     "minecraft is written in java",
    //     "котики это мило",
    //     "собаки умнее котов"
    // ];

    // let mut embeddings: Vec<(&str, Vec<f32>)> = vec![];
    // for i in texts {
    //     embeddings.push(
    //         (i, engine.generate_embedding(i).unwrap())
    //     )
    // }
    
    // let query_test = "cats are better than dogs";
    // let query_embedding = engine.generate_embedding(query_test).unwrap();

    // for (text, vector) in embeddings {
    //     println!("vector for '{}': {}", text, cosine_similarity(&query_embedding, &vector));
    // }
}

// temporary function to calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
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
