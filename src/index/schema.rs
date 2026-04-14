use tantivy::schema::{Field, STORED, Schema, TEXT};

// Fields of index
pub struct IndexSchema {
    pub schema: Schema,
    pub path: Field,
    pub title: Field,
    pub body: Field,
}

impl IndexSchema {
    pub fn new() -> IndexSchema {
        let mut builder = Schema::builder();

        // STORED - save value. Only save, there should not find by it
        // TEXT - divide into words and build index (there is able to find)
        let path = builder.add_text_field("path", STORED);
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
