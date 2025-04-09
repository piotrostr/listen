use anyhow::Result;
use rig::embeddings::EmbeddingModel;

pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let embedder = rig::providers::gemini::Client::from_env()
        .embedding_model(rig::providers::gemini::embedding::EMBEDDING_004);

    let embedding = embedder.embed_text(text).await?;

    Ok(embedding.vec.iter().map(|f| *f as f32).collect())
}
