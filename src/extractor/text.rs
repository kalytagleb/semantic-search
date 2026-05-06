use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "pdf" => pdf_extract::extract_text(path).map_err(|e| e.to_string()),
        "docx" => {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            let doc = docx_rs::read_docx(&bytes).map_err(|e| e.to_string())?;
            // Extract text from paragraphs
            let mut text = String::new();
            for child in doc.document.children {
                if let docx_rs::DocumentChild::Paragraph(p) = child {
                    for child in p.children {
                        if let docx_rs::ParagraphChild::Run(r) = child {
                            for child in r.children {
                                if let docx_rs::RunChild::Text(t) = child {
                                    text.push_str(&t.text);
                                }
                            }
                        }
                    }
                    text.push('\n');
                }
            }
            Ok(text)
        }
        _ => {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_file() {
        let result = read(Path::new("Cargo.toml"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("package"));
    }
}
