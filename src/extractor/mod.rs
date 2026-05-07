pub mod chunker;
pub mod text;

use std::path::Path;

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    text::read(path).map_err(|e| anyhow::anyhow!(e))
}
