use anyhow::Result;
use rig_tool_macro::tool;

pub mod client;
pub mod search;

pub use client::{LunarCrushApiClient, LunarCrushApiResponseError};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub config: Option<ApiConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiConfig {
    pub topic: String,
    pub type_: Option<String>,
    pub id: String,
    pub generated: u64,
}

// LunarCrush API Implementation
pub struct LunarCrushApi {
    pub client: LunarCrushApiClient,
}

#[derive(Debug, thiserror::Error)]
pub enum LunarCrushApiError {
    #[error("[LunarCrushAPI] LunarCrush API Error: {0}")]
    ApiError(LunarCrushApiResponseError),

    #[error("[LunarCrushAPI] Failed to parse response: {0}")]
    ParseError(reqwest::Error),

    #[error("[LunarCrushAPI] Failed to deserialize response: {0}")]
    RequestError(reqwest::Error),

    #[error("[LunarCrushAPI] Deserialize error: {0} body: {1}")]
    DeserializeError(serde_json::Error, String),

    #[error("[LunarCrushAPI] Invalid input: {0}")]
    InvalidInput(anyhow::Error),
}

#[tool(description = "
Research a cryptocurrency or blockchain topic using LunarCrush, which aggregates and analyzes social media sentiment and activity.

Parameters:
- topic (string): The cryptocurrency/blockchain topic to research. For Solana tokens, provide the mint address directly for best results. It could be any tag.

Returns information about:
- Topic overview and metrics
- Social engagement and sentiment analysis
")]
pub async fn analyze_sentiment(topic: String) -> Result<serde_json::Value> {
    let lunarcrush = LunarCrushApi::from_env()?;

    let result = lunarcrush
        .research_topic(&topic)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to research topic: {}", e))?;

    Ok(result)
}

impl LunarCrushApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: LunarCrushApiClient::new(api_key, None),
        }
    }

    pub fn from_env() -> Result<Self> {
        let client = LunarCrushApiClient::new(
            std::env::var("LUNARCRUSH_API_KEY").unwrap(),
            Some("https://lunarcrush.com/api4/public".to_string()),
        );
        Ok(Self { client })
    }

    pub async fn research_topic(
        &self,
        topic: &str,
    ) -> Result<serde_json::Value, LunarCrushApiError> {
        let topic_info = self.fetch_topic_info(&topic).await?;

        // let posts = self.fetch_topic_posts(&topic).await?;

        if std::env::var("RUST_LOG").unwrap_or_default() == "debug" {
            let _ = std::fs::create_dir_all("debug");
            let _ = std::fs::write(
                "debug/lunarcrush_topic.json",
                serde_json::to_string(&topic_info).unwrap(),
            );
            // let _ = std::fs::write(
            //     "debug/lunarcrush_posts.json",
            //     serde_json::to_string(&posts.data).unwrap(),
            // );
        }

        let res = serde_json::json!({
            "topic": topic_info,
            // "posts": posts.data,
        });

        Ok(res)
    }

    pub async fn fetch_topic_info(
        &self,
        topic: &str,
    ) -> Result<serde_json::Value, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/v1", topic);
        let response = self
            .client
            .request::<ApiResponse<serde_json::Value>>(&endpoint, None)
            .await?;
        Ok(response.data)
    }

    pub async fn fetch_topic_posts(
        &self,
        topic: &str,
    ) -> Result<ApiResponse<Vec<serde_json::Value>>, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/posts/v1", topic);
        let response = self
            .client
            .request::<ApiResponse<Vec<serde_json::Value>>>(&endpoint, None)
            .await?;
        Ok(response)
    }

    pub async fn fetch_topic_creators(
        &self,
        topic: &str,
    ) -> Result<ApiResponse<serde_json::Value>, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/creators/v1", topic);
        let response = self
            .client
            .request::<ApiResponse<serde_json::Value>>(&endpoint, None)
            .await?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[timed::timed]
    #[tokio::test]
    async fn lunarcrush_e2e_bitcoin() {
        let lunarcrush = LunarCrushApi::from_env().unwrap();
        let summary = lunarcrush
            .research_topic("Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump")
            .await
            .unwrap();

        println!("{:#?}", summary);
    }
}
