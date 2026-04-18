use crate::scanner::filters::FileType;
use text_splitter::{ChunkConfig, TextSplitter};

pub fn split(text: &str, file_type: &FileType) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let (max_characters, overlap) = match file_type {
        // 600 characters + 150 overlap (25%)
        FileType::Code => (600, 150),

        FileType::Text | FileType::Data => (800, 200),
    };

    let config_chunks = ChunkConfig::new(max_characters)
        .with_overlap(overlap)
        .expect("Overlap must be less than chunk size");

    let splitter = TextSplitter::new(config_chunks);

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

    #[test]
    fn test_anna_karenina() {
        let text = "Анна Каренина — молодая, красивая и статная женщина, жена влиятельного петербургского чиновника Алексея Каренина. \
        Её жизнь кажется идеальной, но внутри она чувствует пустоту. Всё меняется, когда она отправляется в Москву, чтобы помочь брату Стиве Облонскому спасти его брак. \
        На вокзале она встречает Алексея Вронского, блестящего офицера. Эта встреча становится роковой. Между ними вспыхивает всепоглощающая страсть, \
        которая идет вразрез со всеми моральными нормами того времени. Анна решается на открытую связь с Вронским, что приводит к её полному изгнанию из высшего общества. \
        Общество, которое само полно лицемерия, не прощает ей этой честности в чувствах. Каренин, руководствуясь холодным долгом и страхом перед скандалом, отказывает ей в разводе и разлучает с сыном Серёжей. \
        Анна переезжает в поместье к Вронскому, но счастья это не приносит. Её терзают сомнения, ревность и чувство вины. Ей кажется, что любовь Вронского остывает. \
        В это же время мы видим другую линию — Константина Левина. Он, в отличие от Анны и Вронского, ищет смысл жизни не в страсти, а в труде, семье и связи с землёй. \
        Левин проходит через мучительные поиски веры и в итоге находит успокоение в браке с Кити Щербацкой. \
        Финал истории Анны трагичен: запутавшись в своих чувствах и не видя выхода из социального тупика, она бросается под товарный поезд на той самой станции, где впервые увидела Вронского. \
        Этот роман — глубокое исследование человеческой души, столкновения долга и чувства, а также портрет эпохи, где правила общества оказываются сильнее человеческого счастья.";

        let chunks = split(text, &FileType::Text);

        println!("Total text length: {} chars.", text.chars().count());
        println!("Chunks: {}", chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            println!(
                "\n--- Chunk {} (length: {}) ---",
                i + 1,
                chunk.chars().count()
            );
            println!("{}", chunk);

            if i < chunks.len() - 1 {
                println!("End of chunk {}", i + 1);
            }
        }
    }
}
