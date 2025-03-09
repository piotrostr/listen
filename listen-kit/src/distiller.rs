use anyhow::Result;

use rig::{
    completion::Prompt,
    providers::gemini::completion::CompletionModel as GeminiCompletionModel,
};

pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;

pub fn make_distiller_agent() -> Result<GeminiAgent> {
    Ok(rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .preamble("Your job is to extract the most relevant content from an
        Twitter API response and provide a summary. Be sure to take into account
        things like mindshare, the likes, retweets")
        .build())
}

/// Distiller is a wrapper around multimodal Gemini 2.0 that allows to bring
/// understanding of assets, pass it a link to an image, video or large block of
/// text and receive a summary of the content.
pub struct Distiller {
    pub agent: GeminiAgent,
}

#[derive(Debug, thiserror::Error)]
pub enum DistillerError {
    #[error("GEMINI_API_KEY is not set")]
    GeminiApiKeyNotSet,

    #[error("Model error")]
    PromptError(rig::completion::PromptError),
}

impl Distiller {
    pub fn from_env() -> Result<Self, DistillerError> {
        let agent = make_distiller_agent()
            .map_err(|_| DistillerError::GeminiApiKeyNotSet)?;
        Ok(Self { agent })
    }

    // TODO cache this by hash of entire value that goes in to improve latency
    // for subsequent calls
    pub async fn distill(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, DistillerError> {
        self.agent
            .prompt(response.to_string())
            .await
            .map_err(DistillerError::PromptError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distiller() {
        let distiller = Distiller::from_env().unwrap();
        let result = distiller
            .distill(
                &std::fs::read_to_string("./debug/tweets_by_ids.json")
                    .unwrap()
                    .parse::<serde_json::Value>()
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:#?}", result);
    }
}
