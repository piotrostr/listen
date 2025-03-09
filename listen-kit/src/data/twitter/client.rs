use super::TwitterApiError;
use serde::{Deserialize, Serialize};

pub struct TwitterApiClient {
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterApiResponseError {
    pub error: u32,
    pub message: String,
}

impl std::fmt::Display for TwitterApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.error, self.message)
    }
}

impl TwitterApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://api.twitterapi.io".to_string()),
        }
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
        let text =
            response.text().await.map_err(TwitterApiError::ParseError)?;

        tracing::debug!("Twitter API Response: {}", text);

        // Try to parse the JSON
        match serde_json::from_str::<T>(&text) {
            Ok(data) => Ok(data),
            Err(e) => Err(TwitterApiError::DeserializeError(e, text)),
        }
    }
}
