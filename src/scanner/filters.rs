use std::{fs::File, io::Read, path::Path};

pub enum FileType {
    Text,     // .txt .md .html, usual texts, README
    DataCode, // code (js, py) and (json, yaml)
    Doc,      // .pdf .docx
}

fn detect_by_content(path: &Path) -> Option<Option<FileType>> {
    let mut file = File::open(path).ok()?;
    let mut buffer = [0; 16];

    let n = file.read(&mut buffer).ok()?;
    if n == 0 {
        return None;
    }

    let kind = infer::get(&buffer[..n])?;

    match kind.mime_type() {
        "application/pdf"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some(Some(FileType::Doc))
        }
        "application/x-executable" | "application/zip" | "application/x-rar-compressed" => {
            Some(None)
        }
        _ => None,
    }
}

pub fn get_file_type(path: &Path) -> Option<FileType> {
    if let Some(content_res) = detect_by_content(path) {
        return content_res;
    }

    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" | "docx" => Some(FileType::Doc),
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp" | "cs" | "java" | "json"
        | "yaml" | "toml" | "xml" | "csv" | "sh" | "bat" => Some(FileType::DataCode),
        _ => {
            if is_not_binary(path) {
                Some(FileType::Text)
            } else {
                None
            }
        }
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

// Binary file check
fn is_not_binary(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0; 1024];

    match file.read(&mut buffer) {
        Ok(n) => !buffer[..n].contains(&0),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self};
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_datacode_type() {
        assert!(matches!(
            get_file_type(Path::new("main.rs")),
            Some(FileType::DataCode)
        ));
        assert!(matches!(
            get_file_type(Path::new("data.json")),
            Some(FileType::DataCode)
        ));
    }

    #[test]
    fn test_text_and_binary_detection() {
        let text_path = Path::new("test_readme_file");
        {
            let mut file = fs::File::create(text_path).expect("Error.");
            file.write_all(b"Hello, momd").expect("Error.");
        }

        assert!(matches!(get_file_type(text_path), Some(FileType::Text)));
        fs::remove_file(text_path).ok();

        // Test binary file
        let bin_path = Path::new("test_binary_file.dat");
        {
            let mut file = fs::File::create(bin_path).expect("Error.");
            file.write_all(&[0x48, 0x65, 0x6C, 0x6F, 0x00])
                .expect("Error");
        }

        assert!(get_file_type(bin_path).is_none());
        fs::remove_file(bin_path).ok();
    }

    #[test]
    fn test_size_filter() {
        let path = Path::new("test_size_file");
        let file = fs::File::create(path).expect("Error");

        file.set_len(100).expect("Error");
        assert!(is_size_ok(path));

        // 60 MB - a lot (limit 50 MB)
        file.set_len(60 * 1024 * 1024).expect("Error");
        assert!(!is_size_ok(path));

        drop(file);
        fs::remove_file(path).ok();
    }
}
