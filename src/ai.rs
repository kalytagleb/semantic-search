use fastembed::{TextEmbedding, InitOptions};
use anyhow::Context;

// structure to hold our AI model
pub struct AiEngine {
    model: TextEmbedding,
}

impl AiEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        println!("Loading AI model, this may take a moment on first run.");

        let options = InitOptions::new(fastembed::EmbeddingModel::MultilingualE5Small);
        let model = fastembed::TextEmbedding::try_new(options)?;
        return Ok(AiEngine { model });
    }

    pub fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let vec_embed = vec![text];
        self.model.embed(vec_embed, None)?.get(0).cloned().ok_or_else(|| anyhow::anyhow!("Failure"))
    }

    pub fn generate_embeddings_batch(&mut self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, anyhow::Error> {
        self.model.embed(texts, None)
            .context("Failed to generate embeddings for batch")
    }
}