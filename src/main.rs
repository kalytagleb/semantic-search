use std::path::Path;

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
    vector_db::save_db(&records, Path::new("vector_db.bin"));

    let vectors_test = vector_db::load_db(Path::new("vector_db.bin"));
    for myvec in vectors_test {
        println!("{:?}", myvec.chunk_text);

    const DB_PATH: &str = "vectors_db.bin";

    println!("scanning files");
    let files = scanner::scan_all("./my_test_folder");
    println!("Found {} files", files.len());

    if files.is_empty() {
        println!("No files found. Exiting.");
        return;
    }

    println!("\ninitializing AI Engine");
    let mut engine = match ai::AiEngine::new() {
        Ok(e) => {
            println!("AI Engine loaded successfully");
            e
        }
        Err(e) => {
            println!("Failed to load AI Engine: {}", e);
            return;
        }
    };

    println!("processing files");
    let mut all_records: Vec<VectorRecord> = Vec::new();

    for file_path in files {
        println!("Processing: {}", file_path.display());

        // Read file content
        let content = match extractor::text::read(&file_path) {
            Ok(c) => c,
            Err(e) => {
                println!("  Error reading file: {}", e);
                continue;
            }
        };
        println!("  Read {} bytes", content.len());

        // Get file type and split into chunks
        let file_type = match scanner::filters::get_file_type(&file_path) {
            Some(ft) => ft,
            None => {
                println!("  Error: unsupported file type");
                continue;
            }
        };

        let chunks = extractor::chunker::split(&content, &file_type);
        println!("  Split into {} chunks", chunks.len());

        // Generate embeddings for each chunk
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            match engine.generate_embedding(chunk) {
                Ok(vector) => {
                    let record = VectorRecord {
                        path: file_path.to_string_lossy().to_string(),
                        chunk_text: chunk.clone(),
                        vector,
                    };
                    all_records.push(record);
                    println!("    Chunk {}: embedded ({} dims)", chunk_idx + 1, all_records.last().unwrap().vector.len());
                }
                Err(e) => {
                    println!("    Chunk {}: Error generating embedding: {}", chunk_idx + 1, e);
                }
            }
        }
    }

    println!("Saving to database");
    println!("Total records to save: {}", all_records.len());
    match vector_db::save_db(&all_records, Path::new(DB_PATH)) {
        Ok(()) => println!("Saved to {}", DB_PATH),
        Err(e) => {
            eprintln!("Error saving database: {}", e);
            return;
        }
    }

    println!("Loading from database");
    let loaded_records = match vector_db::load_db(Path::new(DB_PATH)) {
        Ok(records) => records,
        Err(e) => {
            eprintln!("Error loading database: {}", e);
            return;
        }
    };
    
    for (i, record) in loaded_records.iter().enumerate().take(5) {
        println!("Record {}: path='{}', chunk_len={}, vector_dims={}", 
            i + 1, 
            record.path, 
            record.chunk_text.len(),
            record.vector.len()
        );
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
