pub mod chunker;
pub mod text;

use crate::scanner::filters;
use std::path::Path;

#[allow(dead_code)]
pub struct ExtractedText {
    pub chunks: Vec<String>, // chunks - for vector index
}

#[allow(dead_code)]
pub fn extract(path: &Path) -> Result<ExtractedText, String> {
    let file_type = match filters::get_file_type(path) {
        Some(ft) => ft,
        None => return Err(format!("Format doesn't support {:?}", path)),
    };

    let content = text::read(path)?;
    let chunks = chunker::split(&content, &file_type);

    Ok(ExtractedText { chunks })
}
