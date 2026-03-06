mod extractor;
mod scanner;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    println!("Scanning: {}", root);

    let files = scanner::scan_all(&root);

    println!("There are: {} files", files.len());

    for file in &files {
        match extractor::extract(file) {
            Ok(extracted) => {
                println!(
                    "[OK] {} - {} characters, {} chunks",
                    file.display(),
                    extracted.content.len(),
                    extracted.chunks.len()
                );
            }
            Err(e) => {
                println!("[ERR] {}: {}", file.display(), e);
            }
        }
    }
}
