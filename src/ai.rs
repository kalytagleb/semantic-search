use anyhow::Context;
use fastembed::{InitOptions, TextEmbedding};

pub struct AiEngine {
    model: TextEmbedding,
}

impl AiEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        let options = InitOptions::new(fastembed::EmbeddingModel::EmbeddingGemma300M);
        let model = fastembed::TextEmbedding::try_new(options)?;
        Ok(AiEngine { model })
    }

    pub fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let vec_embed = vec![text];
        self.model
            .embed(vec_embed, None)?
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Failure"))
    }

    pub fn generate_query_embedding(
        &mut self,
        user_query: &str,
    ) -> Result<Vec<f32>, anyhow::Error> {
        let prompt = format!("query: {}", user_query);
        self.generate_embedding(&prompt)
    }

    pub fn generate_embeddings_batch(
        &mut self,
        texts: Vec<&str>,
    ) -> Result<Vec<Vec<f32>>, anyhow::Error> {
        self.model
            .embed(texts, None)
            .context("Failed to generate embeddings for batch")
    }
}
