use anyhow::Result;

pub mod client;
pub mod search;
pub mod tweets;
pub mod user_info;
pub mod user_tweets;

// Re-export common types
pub use client::{TwitterApiClient, TwitterApiResponseError};
pub use tweets::Tweet;
pub use user_info::UserInfo;
pub use user_tweets::UserTweet;

// Common types shared across modules
use serde::{Deserialize, Serialize};

use crate::twitter::user_tweets::FetchUserTweetsOptions;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: Status,
    pub msg: String,
}

// Twitter API Implementation
pub struct TwitterApi {
    pub client: TwitterApiClient,
}

#[derive(Debug, thiserror::Error)]
pub enum TwitterApiError {
    #[error("[TwitterAPI] Twitter API Error: {0}")]
    ApiError(TwitterApiResponseError),

    #[error("[TwitterAPI] Failed to parse response: {0}")]
    ParseError(reqwest::Error),

    #[error("[TwitterAPI] Failed to deserialize response: {0}")]
    RequestError(reqwest::Error),

    #[error("[TwitterAPI] Deserialize error: {0} body: {1}")]
    DeserializeError(serde_json::Error, String),

    #[error("[TwitterAPI] Invalid input: {0}")]
    InvalidInput(anyhow::Error),
}

impl TwitterApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: TwitterApiClient::new(api_key, None),
        }
    }

    pub fn from_env() -> Result<Self> {
        let client = TwitterApiClient::new(
            std::env::var("TWITTERAPI_API_KEY").unwrap(),
            Some("https://api.twitterapi.io".to_string()),
        );
        Ok(Self { client })
    }

    pub async fn research_profile(
        &self,
        username: &str,
    ) -> Result<serde_json::Value, TwitterApiError> {
        let profile =
            self.fetch_user_info(&username.replace("@", "")).await?;
        let tweets_response = self
            .fetch_user_tweets(FetchUserTweetsOptions {
                user_id: None,
                username: Some(username.to_string()),
                include_replies: Some(false),
                cursor: None,
            })
            .await?;

        if std::env::var("RUST_LOG").unwrap_or_default() == "debug" {
            std::fs::write(
                "debug/profile.json",
                serde_json::to_string(&profile).unwrap(),
            )
            .expect("failed to write debug output");
            std::fs::write(
                "debug/tweets.json",
                serde_json::to_string(&tweets_response.tweets).unwrap(),
            )
            .expect("failed to write debug output");
        }

        let res = serde_json::json!({
            "profile": profile,
            "tweets": tweets_response.tweets,
        });

        Ok(res)
    }
}

// Summary types for combined data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetSummary {
    pub text: String,
    pub author_username: Option<String>,
    pub created_at: String,
    pub url: Option<String>,
    pub retweet_count: Option<u32>,
    pub reply_count: Option<u32>,
    pub like_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfileResearch {
    pub profile: UserInfo,
    pub tweets: Vec<TweetSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[timed::timed]
    #[tokio::test]
    async fn twitter_e2e_listen() {
        let twitter = TwitterApi::from_env().unwrap();
        let summary = twitter.research_profile("listenonsol").await.unwrap();

        println!("{:#?}", summary);
    }

    #[timed::timed]
    #[tokio::test]
    async fn twitter_e2e_arc() {
        let twitter = TwitterApi::from_env().unwrap();
        let summary = twitter.research_profile("arcdotfun").await.unwrap();

        println!("{:#?}", summary);
    }
}
