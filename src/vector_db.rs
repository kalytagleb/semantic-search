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
    println!("Serialized: {:?}", serialized);
    fs::write(path, serialized).expect("Could not write file");
}

pub fn load_db(path: &std::path::Path) -> Vec<VectorRecord> {
    let serialized = fs::read(path).expect("Could not read file");
    let deserialized = bincode::deserialize(&serialized).unwrap();
    println!("Deserialized: {:?}", deserialized);
    deserialized
}