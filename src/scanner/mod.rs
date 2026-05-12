pub mod filters;

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use filters::{is_indexable, is_size_ok};

pub fn scan_all(root: &str) -> Vec<PathBuf> {
    let walker = WalkBuilder::new(root)
        .follow_links(false)
        .hidden(true) // Miss all hidden files (starts with .)
        .git_ignore(true)
        .git_global(true)
        .build();

    walker
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if is_system_ignored(path) {
                return None;
            }

            let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);

            if is_file && is_indexable(path) && is_size_ok(path) {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect()
}

fn is_system_ignored(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    let system_roots = [
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Program Files (x86)",
    ];

    #[cfg(not(target_os = "windows"))]
    let system_roots = [
        "/proc",
        "/sys",
        "/dev",
        "/run",
        "/tmp",
        "/var/lib",
        "/var/cache",
    ];

    system_roots.iter().any(|root| path.starts_with(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_folders_are_ignored() {
        #[cfg(not(target_os = "windows"))]
        {
            assert!(is_system_ignored(Path::new("/proc/cpuinfo")));
            assert!(is_system_ignored(Path::new("/sys/kernel")));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(is_system_ignored(Path::new("C:\\Windows\\system32")));
        }
    }

    #[test]
    fn usual_folders_dont_ignore() {
        assert!(!is_system_ignored(Path::new("src/main.rs")));
        assert!(!is_system_ignored(Path::new("my_project/target/file.txt")));
    }
}
