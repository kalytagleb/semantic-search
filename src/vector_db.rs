use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VectorRecord {
    pub path: String,
    pub chunk_text: String,
    pub vector: Vec<f32>,
}

pub fn save_db(records: &[VectorRecord], path: &std::path::Path) -> anyhow::Result<()> {
    let serialized = bincode::serialize(records).context("Failed to serialize vector records")?;

    fs::write(path, serialized).context("Failed to write vectors.bin to disk")?;

    Ok(())
}

pub fn load_db(path: &std::path::Path) -> anyhow::Result<Vec<VectorRecord>> {
    let serialized = fs::read(path).context("Failed to read vectors.bin from disk")?;

    let deserialized =
        bincode::deserialize(&serialized).context("Failed to deserialize vector records")?;

    Ok(deserialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_path(name: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/test_{}.bin", name))
    }

    #[test]
    fn test_save_and_load_single_record() {
        let temp_path = create_temp_path("single_record");
        let records = vec![VectorRecord {
            path: "test.txt".to_string(),
            chunk_text: "Hello world".to_string(),
            vector: vec![0.1, 0.2, 0.3],
        }];

        // Save
        assert!(save_db(&records, &temp_path).is_ok());
        assert!(temp_path.exists());

        // Load
        let loaded = load_db(&temp_path).expect("Failed to load");
        assert_eq!(loaded, records);

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_save_and_load_multiple_records() {
        let temp_path = create_temp_path("multiple_records");
        let records = vec![
            VectorRecord {
                path: "file1.txt".to_string(),
                chunk_text: "First chunk".to_string(),
                vector: vec![1.0, 2.0, 3.0],
            },
            VectorRecord {
                path: "file2.txt".to_string(),
                chunk_text: "Second chunk".to_string(),
                vector: vec![4.0, 5.0, 6.0],
            },
            VectorRecord {
                path: "file3.txt".to_string(),
                chunk_text: "Third chunk with Russian: привет".to_string(),
                vector: vec![7.0, 8.0, 9.0],
            },
        ];

        save_db(&records, &temp_path).expect("Failed to save");
        let loaded = load_db(&temp_path).expect("Failed to load");

        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded, records);

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_path = create_temp_path("nonexistent");
        let result = load_db(&temp_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn test_save_empty_records() {
        let temp_path = create_temp_path("empty");
        let records: Vec<VectorRecord> = vec![];

        assert!(save_db(&records, &temp_path).is_ok());
        let loaded = load_db(&temp_path).expect("Failed to load");
        assert_eq!(loaded.len(), 0);

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_large_vectors() {
        let temp_path = create_temp_path("large_vectors");
        let large_vector = (0..384).map(|i| i as f32 / 384.0).collect();
        let records = vec![VectorRecord {
            path: "large.txt".to_string(),
            chunk_text: "Large vector test".to_string(),
            vector: large_vector,
        }];

        save_db(&records, &temp_path).expect("Failed to save");
        let loaded = load_db(&temp_path).expect("Failed to load");

        assert_eq!(loaded[0].vector.len(), 384);
        assert_eq!(loaded, records);

        fs::remove_file(temp_path).ok();
    }
}
