use tantivy::schema::{Field, INDEXED, STORED, STRING, Schema, TEXT};

pub struct IndexSchema {
    pub schema: Schema,
    pub path: Field,
    pub path_tokens: Field,
    pub extension: Field,
    pub last_modified: Field,
    pub chunk_index: Field,
    pub text: Field,
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexSchema {
    pub fn new() -> Self {
        let mut builder = Schema::builder();
        let path = builder.add_text_field("path", STRING | STORED);
        let path_tokens = builder.add_text_field("path_tokens", TEXT | STORED);
        let extension = builder.add_text_field("extension", STRING | STORED);
        let last_modified = builder.add_u64_field("last_modified", INDEXED | STORED);
        let chunk_index = builder.add_u64_field("chunk_index", STORED);
        let text = builder.add_text_field("text", TEXT | STORED);

        Self {
            schema: builder.build(),
            path,
            path_tokens,
            extension,
            last_modified,
            chunk_index,
            text,
        }
    }
}
