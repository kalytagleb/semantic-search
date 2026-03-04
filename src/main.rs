mod scanner;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    println!("Our target directory: {}", root);

    let files = scanner::find_indicated_files(&root);

    println!("There are: {} files", files.len());

    for file in &files {
        println!(" {}", file.display());
    }
}
