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
    // System virtual filesystems — scanning these hangs or causes errors
    let system_roots = ["/proc", "/sys", "/dev", "/run", "/boot", "/tmp"];
    let path_str = path.to_string_lossy();
    if system_roots.iter().any(|r| path_str.starts_with(r)) {
        return true;
    }

    let ignored_names = [
        "target",
        "node_modules",
        "venv",
        "__pycache__",
        "build",
        "dist",
    ];

    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        // Skip known noisy dirs
        if ignored_names.contains(&name.as_ref()) {
            return true;
        }
        // Skip hidden files and folders (starting with . but not . or ..)
        if name.starts_with('.') && name != "." && name != ".." {
            return true;
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_folder_ignores() {
        assert!(is_ignored(Path::new("target/debug/build/file.json")));
        assert!(is_ignored(Path::new("node_modules/main.js")));
        assert!(is_ignored(Path::new("/proc/cpuinfo")));
        assert!(is_ignored(Path::new("/sys/kernel/mm")));
        assert!(is_ignored(Path::new("some/.hidden_folder/file.txt")));
        assert!(is_ignored(Path::new(".hidden_file.txt")));
    }

    #[test]
    fn usual_folders_dont_ignore() {
        assert!(!is_ignored(Path::new("src/main.rs")));
        assert!(!is_ignored(Path::new("src/scanner/mod.rs")));
        assert!(!is_ignored(Path::new("/home/user/Documents/report.pdf")));
    }
}
