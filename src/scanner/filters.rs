use std::path::Path;

fn supported_extensions() -> Vec<String> {
    vec![
        // Text and documents
        "txt".to_string(),
        "md".to_string(),
        "pdf".to_string(),
        "docx".to_string(),
        "html".to_string(),
        // Code
        "rs".to_string(),
        "py".to_string(),
        "js".to_string(),
        "ts".to_string(),
        "go".to_string(),
        "cpp".to_string(),
        "java".to_string(),
        // Configs
        "yaml".to_string(),
        "json".to_string(),
        "toml".to_string(),
        "csv".to_string(),
    ]
}

// Defend Extractor from unsuitable formats
pub fn is_indexable(path: &Path) -> bool {
    let extension = match path.extension() {
        Some(ext) => ext.to_string_lossy().to_lowercase(),
        None => return false,
    };

    let extensions = supported_extensions();
    extensions.contains(&extension.to_string())
}

// Program doesn't accept files more than 50 MB
pub fn is_size_ok(path: &Path) -> bool {
    const MAX_SIZE: u64 = 50 * 1024 * 1024; // 50 MB

    match path.metadata() {
        Ok(metadata) => metadata.len() <= MAX_SIZE,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn accept_supported_formats() {
        assert!(is_indexable(Path::new("main.rs")));
        assert!(is_indexable(Path::new("main.md")));
        assert!(is_indexable(Path::new("main.toml")));
        assert!(is_indexable(Path::new("main.json")));
        assert!(is_indexable(Path::new("main.py")));
        assert!(is_indexable(Path::new("main.js")));
    }

    #[test]
    fn unaccept_unsupported_formats() {
        assert!(!is_indexable(Path::new("main.png")));
        assert!(!is_indexable(Path::new("main.zip")));
        assert!(!is_indexable(Path::new("main.jpg")));
        assert!(!is_indexable(Path::new("main.exe")));
        assert!(!is_indexable(Path::new("main.mp4")));
        assert!(!is_indexable(Path::new("main.zov")));
    }

    #[test]
    fn file_without_extensions() {
        assert!(!is_indexable(Path::new("Makefile")));
        assert!(!is_indexable(Path::new("Dockerfile")));
        assert!(!is_indexable(Path::new("README")));
    }

    #[test]
    fn case_sensitive_doesnot_important() {
        assert!(is_indexable(Path::new("MAIN.JS")));
        assert!(is_indexable(Path::new("Script.PY")));
        assert!(is_indexable(Path::new("README.MD")));
    }
}
