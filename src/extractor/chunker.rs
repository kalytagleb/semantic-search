use crate::scanner::filters::FileType;
use text_splitter::TextSplitter;

pub fn split(text: &str, file_type: &FileType) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let chunk_range = match file_type {
        // as usual, functions have 100-400 symbols
        // so chunk_size = 400 - is optimal
        FileType::Code => 300..=400,

        // chunk_size = 1000 - keeps related paragraphs in the same chunk
        FileType::Text | FileType::Data => 800..=1000,
    };

    let splitter = TextSplitter::new(chunk_range);

    splitter.chunks(text).map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::filters::FileType;

    #[test]
    fn empty_text_returns_empty_list() {
        let result = split("", &FileType::Text);
        assert!(result.is_empty());
    }

    #[test]
    fn short_text_returns_one_chunk() {
        let text = "Short story about my friend.";
        let chunks = split(text, &FileType::Text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
