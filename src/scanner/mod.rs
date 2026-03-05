pub mod filters;

use std::path::PathBuf;
use walkdir::WalkDir;

use filters::{is_indexable, is_size_ok};

pub fn scan_all(root: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_type().is_symlink())
        .filter(|e| is_indexable(e.path()))
        .filter(|e| is_size_ok(e.path()))
        .map(|e| e.into_path())
        .collect()
}
