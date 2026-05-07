use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};

pub struct IndexSchema {
    pub schema: Schema,
    pub path: Field,
    pub extension: Field,
    pub chunk_index: Field,
    pub text: Field,
}

impl IndexSchema {
    pub fn new() -> Self {
        let mut builder = Schema::builder();
        let path = builder.add_text_field("path", STRING | STORED);
        let extension = builder.add_text_field("extension", STRING | STORED);
        let chunk_index = builder.add_u64_field("chunk_index", STORED);
        let text = builder.add_text_field("text", TEXT | STORED);

        Self {
            schema: builder.build(),
            path,
            extension,
            chunk_index,
            text,
        }
    }
}
