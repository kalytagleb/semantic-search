use rust_project_52_teamvin::vector_db::{VectorDb, VectorRecord};
use std::path::Path;

#[test]
fn test_cosine_similarity_identical_vectors() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![1.0, 2.0, 3.0];

    let score = rust_project_52_teamvin::math::cosine_similarity(&a, &b);
    assert!(
        (score - 1.0).abs() < 1e-6,
        "Identical vectors should have similarity 1.0"
    );
}

#[test]
fn test_cosine_similarity_orthogonal_vectors() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];

    let score = rust_project_52_teamvin::math::cosine_similarity(&a, &b);
    assert!(
        score.abs() < 1e-6,
        "Orthogonal vectors should have similarity close to 0.0"
    );
}

#[test]
fn test_vector_db_insert_and_load() {
    let db_path = Path::new("/tmp/test_integration_vectors.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).ok();
    }

    let db = VectorDb::open(db_path).expect("Failed to open database");

    let records = vec![
        VectorRecord {
            path: "docs/readme.md".to_string(),
            chunk_index: 0,
            chunk_text: "Hello world".to_string(),
            vector: vec![0.1, 0.2, 0.3],
            content_hash: "hash1".to_string(),
        },
        VectorRecord {
            path: "src/main.rs".to_string(),
            chunk_index: 0,
            chunk_text: "fn main() {}".to_string(),
            vector: vec![0.4, 0.5, 0.6],
            content_hash: "hash2".to_string(),
        },
    ];

    for r in &records {
        db.insert_chunk(r).expect("Failed to insert chunk");
    }

    assert_eq!(db.chunk_count().unwrap(), 2);

    let loaded = db.load_all_vectors().expect("Failed to load vectors");
    assert_eq!(loaded.len(), 2);

    std::fs::remove_file(db_path).ok();
}

#[test]
fn test_incremental_indexing_skips_unchanged_files() {
    let db_path = Path::new("/tmp/test_integration_incremental.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).ok();
    }

    let db = VectorDb::open(db_path).expect("Failed to open database");

    db.insert_chunk(&VectorRecord {
        path: "notes.txt".to_string(),
        chunk_index: 0,
        chunk_text: "some notes".to_string(),
        vector: vec![0.1],
        content_hash: "abc".to_string(),
    })
    .unwrap();

    // Same hash → no reindex needed
    assert!(!db.file_needs_reindex("notes.txt", "abc").unwrap());
    // Different hash → reindex needed
    assert!(db.file_needs_reindex("notes.txt", "xyz").unwrap());
    // New file → reindex needed
    assert!(db.file_needs_reindex("new_file.txt", "any").unwrap());

    std::fs::remove_file(db_path).ok();
}
