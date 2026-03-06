pub mod chunker;
pub mod text;

use crate::scanner::filters;
use std::path::Path;

pub struct ExtractedText {
    pub content: String,     // full text - for keyword search
    pub chunks: Vec<String>, // chunks - for vector index
}

pub fn extract(path: &Path) -> Result<ExtractedText, String> {
    let file_type = match filters::get_file_type(path) {
        Some(ft) => ft,
        None => return Err(format!("Format doesn't support {:?}", path)),
    };

    let content = text::read(path)?;
    let chunks = chunker::split(&content, &file_type);

    Ok(ExtractedText { content, chunks })
}
