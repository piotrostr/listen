use anyhow::{anyhow, Result};

use crate::data::twitter_types::*;

pub struct TwitterApiClient {
    api_key: String,
    base_url: String,
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
}

impl TwitterApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://api.twitterapi.io".to_string()),
        }
    }

    pub fn from_env() -> Self {
        Self::new(
            std::env::var("TWITTERAPI_API_KEY").unwrap(),
            Some("https://api.twitterapi.io".to_string()),
        )
    }

    pub async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<T, TwitterApiError> {
        let client = reqwest::Client::new();
        let mut url = format!("{}{}", self.base_url, endpoint);

        // Add query parameters if provided
        if let Some(params) = params {
            if !params.is_empty() {
                url.push('?');
                let param_strings: Vec<String> = params
                    .iter()
                    .filter(|(_, v)| !v.is_empty())
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                url.push_str(&param_strings.join("&"));
            }
        }

        let response = client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .map_err(TwitterApiError::RequestError)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();

            let error =
                serde_json::from_str::<TwitterApiResponseError>(&error_text)
                    .map_err(|e| {
                        TwitterApiError::DeserializeError(e, error_text)
                    })?;

            return Err(TwitterApiError::ApiError(error));
        }

        // Get the response text first so we can inspect it on error
        let text = response
            .text()
            .await
            .map_err(|e| TwitterApiError::ParseError(e))?;

        // Try to parse the JSON
        match serde_json::from_str::<T>(&text) {
            Ok(data) => Ok(data),
            Err(e) => Err(TwitterApiError::DeserializeError(e, text)),
        }
    }
}

// Twitter API Implementation
pub struct TwitterApi {
    client: TwitterApiClient,
}

impl TwitterApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: TwitterApiClient::new(api_key, None),
        }
    }

    // Fetch user profile information
    pub async fn fetch_profile(&self, username: &str) -> Result<UserInfo> {
        let mut params = std::collections::HashMap::new();
        params.insert("userName".to_string(), username.to_string());

        let response = self
            .client
            .request::<ApiResponse<UserInfo>>(
                "/twitter/user/info",
                Some(params),
            )
            .await?;

        Ok(response.data)
    }

    // Fetch user's tweets
    pub async fn fetch_posts(
        &self,
        options: FetchPostsOptions,
    ) -> Result<TweetsResponse> {
        if options.user_id.is_none() && options.username.is_none() {
            return Err(anyhow!(
                "Either user_id or username must be provided"
            ));
        }

        let mut params = std::collections::HashMap::new();

        if let Some(user_id) = options.user_id {
            params.insert("userId".to_string(), user_id);
        }

        if let Some(username) = options.username {
            params.insert("userName".to_string(), username);
        }

        if let Some(include_replies) = options.include_replies {
            params.insert(
                "includeReplies".to_string(),
                include_replies.to_string(),
            );
        }

        if let Some(cursor) = options.cursor {
            params.insert("cursor".to_string(), cursor);
        }

        let response = self
            .client
            .request::<ApiResponse<TweetsResponse>>(
                "/twitter/user/last_tweets",
                Some(params),
            )
            .await?;

        Ok(response.data)
    }

    // Get tweets by IDs
    pub async fn get_tweets_by_ids(
        &self,
        tweet_ids: Vec<String>,
    ) -> Result<TweetsResponse> {
        if tweet_ids.is_empty() {
            return Err(anyhow!("At least one tweet ID must be provided"));
        }

        let mut params = std::collections::HashMap::new();
        params.insert("tweet_ids".to_string(), tweet_ids.join(","));

        let response = self
            .client
            .request::<ApiResponse<TweetsResponse>>(
                "/twitter/tweets",
                Some(params),
            )
            .await?;

        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_twitter_e2e() {
        let twitter =
            TwitterApi::new(std::env::var("TWITTERAPI_API_KEY").unwrap());
        let profile = twitter.fetch_profile("listenonsol").await.unwrap();

        let posts = twitter
            .fetch_posts(FetchPostsOptions {
                user_id: None,
                username: Some("listenonsol".to_string()),
                include_replies: Some(false),
                cursor: None,
            })
            .await
            .unwrap();
    }
}
