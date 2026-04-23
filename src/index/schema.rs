use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};

pub struct IndexSchema {
    pub schema: Schema,
    pub path: Field,
    pub title: Field,
    pub body: Field,
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexSchema {
    pub fn new() -> IndexSchema {
        let mut builder = Schema::builder();

        // STRING = indexed but not tokenized (exact match, needed for deletion by path)
        let path = builder.add_text_field("path", STRING | STORED);
        let title = builder.add_text_field("title", TEXT | STORED);
        let body = builder.add_text_field("body", TEXT | STORED);

        IndexSchema {
            schema: builder.build(),
            path,
            title,
            body,
        }
    }
}
