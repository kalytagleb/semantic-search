use std::fs;
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorRecord {
    pub path: String,
    pub chunk_text: String,
    pub vector: Vec<f32>,
}

pub fn save_db(records: &Vec<VectorRecord>, path: &std::path::Path) {
    let serialized = bincode::serialize(&records).unwrap();
    // println!("Serialized: {:?}", serialized);
    fs::write(path, serialized).expect("Could not write file");
}

pub fn load_db(path: &std::path::Path) -> Vec<VectorRecord> {
    // Check if file exists and has content
    if !path.exists() {
        println!("Database file not found: {:?}", path);
        return Vec::new();
    }

    let serialized = match fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            println!("Error reading file: {}", e);
            return Vec::new();
        }
    };

    // Check if file is empty
    if serialized.is_empty() {
        println!("Database file is empty");
        return Vec::new();
    }

    match bincode::deserialize(&serialized) {
        Ok(deserialized) => {
            // println!("Deserialized: {:?}", deserialized);
            deserialized
        }
        Err(e) => {
            println!("Error deserializing: {}", e);
            Vec::new()
        }
    }
}