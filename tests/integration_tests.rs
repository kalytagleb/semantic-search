use std::path::Path;

#[test]
fn test_indexing_creates_database() {
    let db_path = Path::new("vectors_db.bin");
    
    if db_path.exists() {
        std::fs::remove_file(db_path).ok();
    }
    
    let records = rust_project_52_teamvin::indexing::run("./my_test_folder")
        .expect("Indexing failed");
    
    assert!(!records.is_empty(), "Should have indexed some records");
    assert!(db_path.exists(), "Database file should be created");
    
    let loaded = rust_project_52_teamvin::vector_db::load_db(db_path)
        .expect("Failed to load database");
    
    assert_eq!(records.len(), loaded.len(), "Record counts should match");
}

#[test]
fn test_cosine_similarity_identical_vectors() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![1.0, 2.0, 3.0];
    
    let score = rust_project_52_teamvin::math::cosine_similarity(&a, &b);
    assert!((score - 1.0).abs() < 1e-6, "Identical vectors should have similarity 1.0");
}

#[test]
fn test_cosine_similarity_orthogonal_vectors() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    
    let score = rust_project_52_teamvin::math::cosine_similarity(&a, &b);
    assert!(score.abs() < 1e-6, "Orthogonal vectors should have similarity close to 0.0");
}

#[test]
fn test_vector_db_save_and_load() {
    let test_path = Path::new("/tmp/test_vectors.bin");
    
    if test_path.exists() {
        std::fs::remove_file(test_path).ok();
    }
    
    let records = vec![
        rust_project_52_teamvin::VectorRecord {
            path: "test1.txt".to_string(),
            chunk_text: "Hello world".to_string(),
            vector: vec![0.1, 0.2, 0.3],
        },
        rust_project_52_teamvin::VectorRecord {
            path: "test2.txt".to_string(),
            chunk_text: "Goodbye world".to_string(),
            vector: vec![0.4, 0.5, 0.6],
        },
    ];
    
    rust_project_52_teamvin::vector_db::save_db(&records, test_path)
        .expect("Failed to save database");
    
    assert!(test_path.exists(), "Database file should exist");
    
    let loaded = rust_project_52_teamvin::vector_db::load_db(test_path)
        .expect("Failed to load database");
    
    assert_eq!(records.len(), loaded.len(), "Should load same number of records");
    
    std::fs::remove_file(test_path).ok();
}
