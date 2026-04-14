pub mod filters;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use filters::{is_indexable, is_size_ok};

pub fn scan_all(root: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_ignored(e.path()))
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_type().is_symlink())
        .filter(|e| is_indexable(e.path()))
        .filter(|e| is_size_ok(e.path()))
        .map(|e| e.into_path())
        .collect()
}

fn is_ignored(path: &Path) -> bool {
    let ignored = [
        "target",
        ".git",
        "node_modules",
        ".cache",
        "venv",
        "__pycache__",
        ".idea",
        ".vscode",
        "build",
        "dist",
    ];

    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        ignored.contains(&name.as_ref())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_folder_ignores() {
        assert!(is_ignored(Path::new("target/debug/build/file.json")));
        assert!(is_ignored(Path::new(".git/config")));
        assert!(is_ignored(Path::new("node_modules/main.js")));
    }

    #[test]
    fn usual_folders_dont_ignore() {
        assert!(!is_ignored(Path::new("src/main.rs")));
        assert!(!is_ignored(Path::new("src/scanner/mod.rs")));
    }
}
