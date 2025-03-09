use anyhow::Result;
use reqwest::Client;

/// Distiller is a wrapper around multimodal Gemini 2.0 that allows to bring
/// understanding of assets, pass it a link to an image, video or large block of
/// text and receive a summary of the content.
pub struct Distiller {
    pub client: Client,
    pub api_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DistillerError {
    #[error("GEMINI_API_KEY is not set")]
    GeminiApiKeyNotSet,
}

impl Distiller {
    pub fn from_env() -> Result<Self, DistillerError> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| DistillerError::GeminiApiKeyNotSet)?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    // TODO
    pub async fn distill(&self, url: &str) -> Result<String, DistillerError> {
        Ok(url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distiller() {
        let distiller = Distiller::from_env().unwrap();
        let sample_video = "";
        let result = distiller.distill(&sample_video).await.unwrap();
        println!("{:#?}", result);
    }
}
