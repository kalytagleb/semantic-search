use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_all(root: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|format| format.file_type().is_file() && !format.file_type().is_symlink())
        .filter(|format| is_indexable(format.path()))
        .map(|format| format.into_path())
        .collect()
}

fn is_indexable(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| {
            matches!(
                s.to_lowercase().as_str(),
                "txt"
                    | "md"
                    | "pdf"
                    | "rs"
                    | "py"
                    | "js"
                    | "ts"
                    | "go"
                    | "c"
                    | "cpp"
                    | "java"
                    | "yaml"
                    | "json"
                    | "toml"
                    | "csv"
                    | "docx"
                    | "html"
            )
        })
        .unwrap_or(false)
}
