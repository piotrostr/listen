use anyhow::Result;
use rig::{embeddings::EmbeddingModel, providers::openai};
use serde_json::json;
use std::env;

pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let embedder = rig::providers::gemini::Client::from_env()
        .embedding_model(rig::providers::gemini::embedding::EMBEDDING_004);

    let embedding = embedder.embed_text(text).await?;

    Ok(embedding.vec.iter().map(|f| *f as f32).collect())
}

pub async fn generate_embeddings_raw(text: &str, model: &str) -> Result<Vec<f32>> {
    let api_key = env::var("GEMINI_API_KEY")?;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            model, api_key
        ))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": format!("models/{}", model),
            "content": {
                "parts": [{
                    "text": text
                }]
            },
            "taskType": "SEMANTIC_SIMILARITY",
            "outputDimensionality": 768
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let embedding = response_json["embedding"]["values"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();

    Ok(embedding)
}

pub async fn generate_embeddings_openai(text: &str) -> Result<Vec<f32>> {
    let embedder = rig::providers::openai::Client::from_env()
        .embedding_model(openai::embedding::TEXT_EMBEDDING_3_SMALL);

    let embedding = embedder.embed_text(text).await?;

    Ok(embedding.vec.iter().map(|f| *f as f32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[timed::timed]
    #[tokio::test]
    async fn test_generate_embedding() {
        dotenv::dotenv().ok();
        let embedding = generate_embedding("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    // 1.5s latency +-
    #[timed::timed]
    #[tokio::test]
    async fn test_generate_embeddings_raw() {
        dotenv::dotenv().ok();
        let embedding = generate_embeddings_raw("Hello, world!", "gemini-embedding-exp-03-07")
            .await
            .unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[timed::timed]
    #[tokio::test]
    async fn test_generate_embeddings_openai() {
        dotenv::dotenv().ok();
        let embedding = generate_embeddings_openai("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 1536);
    }
}
