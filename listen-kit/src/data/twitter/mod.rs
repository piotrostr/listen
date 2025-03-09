use anyhow::Result;

pub mod client;
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

use crate::data::twitter::user_tweets::FetchUserTweetsOptions;

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
    client: TwitterApiClient,
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
    InvalidInput(String),
}

impl TwitterApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: TwitterApiClient::new(api_key, None),
        }
    }

    pub fn from_env() -> Self {
        let client = TwitterApiClient::new(
            std::env::var("TWITTERAPI_API_KEY").unwrap(),
            Some("https://api.twitterapi.io".to_string()),
        );
        Self { client }
    }

    pub async fn research_profile(
        &self,
        username: &str,
    ) -> Result<UserProfileResearch> {
        let profile = self.fetch_user_info(username).await?;
        let posts = self
            .fetch_user_tweets(FetchUserTweetsOptions {
                user_id: None,
                username: Some(username.to_string()),
                include_replies: Some(false),
                cursor: None,
            })
            .await?;

        let tweets = posts
            .tweets
            .iter()
            .map(|tweet| TweetSummary {
                text: tweet.text.clone(),
                author_username: tweet
                    .author
                    .as_ref()
                    .map(|a| a.user_name.clone()),
                created_at: tweet.created_at.clone(),
                url: tweet.url.clone(),
                retweet_count: tweet.retweet_count,
                reply_count: tweet.reply_count,
                like_count: tweet.like_count,
            })
            .collect::<Vec<_>>();

        Ok(UserProfileResearch { profile, tweets })
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

    #[tokio::test]
    #[ignore]
    async fn twitter_e2e() {
        let twitter = TwitterApi::from_env();
        let summary = twitter.research_profile("listenonsol").await.unwrap();

        println!("{:#?}", summary);
    }
}
