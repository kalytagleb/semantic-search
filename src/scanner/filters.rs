use std::path::Path;

pub enum FileType {
    Text, // .txt .md .html
    Code, // .rs .py .js
    Data, // .json .yaml .csv .toml
}

pub fn get_file_type(path: &Path) -> Option<FileType> {
    let extension = match path.extension() {
        Some(ext) => ext.to_string_lossy().to_lowercase(),
        None => return None,
    };

    match extension.as_str() {
        "txt" | "md" | "html" => Some(FileType::Text),
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" => Some(FileType::Code),
        "json" | "yaml" | "toml" | "csv" => Some(FileType::Data),
        _ => None,
    }
}

// Defend Extractor from unsuitable formats
pub fn is_indexable(path: &Path) -> bool {
    get_file_type(path).is_some()
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
    fn text_format_true() {
        assert!(matches!(
            get_file_type(Path::new("notes.md")),
            Some(FileType::Text)
        ));
        assert!(matches!(
            get_file_type(Path::new("notes.txt")),
            Some(FileType::Text)
        ));
        assert!(matches!(
            get_file_type(Path::new("notes.html")),
            Some(FileType::Text)
        ));
    }

    #[test]
    fn code_format_true() {
        assert!(matches!(
            get_file_type(Path::new("main.rs")),
            Some(FileType::Code)
        ));
        assert!(matches!(
            get_file_type(Path::new("script.py")),
            Some(FileType::Code)
        ));
        assert!(matches!(
            get_file_type(Path::new("index.js")),
            Some(FileType::Code)
        ));
    }

    #[test]
    fn data_format_true() {
        assert!(matches!(
            get_file_type(Path::new("config.toml")),
            Some(FileType::Data)
        ));
        assert!(matches!(
            get_file_type(Path::new("data.json")),
            Some(FileType::Data)
        ));
        assert!(matches!(
            get_file_type(Path::new("table.csv")),
            Some(FileType::Data)
        ));
    }

    #[test]
    fn case_sensitive_doesnot_important() {
        assert!(is_indexable(Path::new("MAIN.JS")));
        assert!(is_indexable(Path::new("Script.PY")));
        assert!(is_indexable(Path::new("README.MD")));
    }
}
