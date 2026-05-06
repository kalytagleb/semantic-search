use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
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
