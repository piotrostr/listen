use super::TwitterApiError;
use serde::{Deserialize, Serialize};

const TWITTERAPI_IO_BASE_URL: &str = "https://api.twitterapi.io";
const XQUIK_BASE_URL: &str = "https://xquik.com/api/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwitterApiProvider {
    TwitterApiIo,
    Xquik,
}

impl TwitterApiProvider {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::TwitterApiIo => TWITTERAPI_IO_BASE_URL,
            Self::Xquik => XQUIK_BASE_URL,
        }
    }

    fn api_key_header(&self) -> &'static str {
        match self {
            Self::TwitterApiIo => "X-API-Key",
            Self::Xquik => "x-api-key",
        }
    }

    fn from_base_url(base_url: &str) -> Self {
        let is_xquik = reqwest::Url::parse(base_url)
            .ok()
            .and_then(|url| {
                url.host_str()
                    .map(|host| host.eq_ignore_ascii_case("xquik.com"))
            })
            .unwrap_or(false);

        if is_xquik {
            Self::Xquik
        } else {
            Self::TwitterApiIo
        }
    }
}

pub(crate) fn xquik_user_identifier(
    identifier: &str,
) -> Result<&str, TwitterApiError> {
    let identifier = identifier.strip_prefix('@').unwrap_or(identifier);
    let is_valid = !identifier.is_empty()
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

    if !is_valid {
        return Err(TwitterApiError::InvalidInput(anyhow::anyhow!(
            "Invalid X username or user ID. Use letters, numerals, or underscores."
        )));
    }

    Ok(identifier)
}

pub struct TwitterApiClient {
    api_key: String,
    base_url: String,
    provider: TwitterApiProvider,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterApiResponseError {
    pub error: serde_json::Value,
    pub message: Option<String>,
}

impl std::fmt::Display for TwitterApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self
            .error
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| self.error.to_string());
        let message = self.message.as_deref().unwrap_or("request failed");
        write!(f, "Error {}: {}", code, message)
    }
}

impl TwitterApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let base_url =
            base_url.unwrap_or_else(|| TWITTERAPI_IO_BASE_URL.to_string());
        Self {
            api_key,
            provider: TwitterApiProvider::from_base_url(&base_url),
            base_url,
        }
    }

    pub fn with_provider(
        api_key: String,
        provider: TwitterApiProvider,
        base_url: Option<String>,
    ) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| provider.base_url().into()),
            provider,
        }
    }

    pub fn new_xquik(api_key: String) -> Self {
        Self::with_provider(api_key, TwitterApiProvider::Xquik, None)
    }

    pub fn provider(&self) -> TwitterApiProvider {
        self.provider
    }

    pub(crate) fn request_url(
        &self,
        endpoint: &str,
        params: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<reqwest::Url, TwitterApiError> {
        let mut url =
            reqwest::Url::parse(&format!("{}{}", self.base_url, endpoint))
                .map_err(|e| {
                    TwitterApiError::InvalidInput(anyhow::anyhow!(
                        "Invalid API URL: {e}"
                    ))
                })?;

        if let Some(params) = params {
            if !params.is_empty() {
                url.query_pairs_mut().extend_pairs(
                    params.iter().filter(|(_, value)| !value.is_empty()),
                );
            }
        }

        Ok(url)
    }

    fn request_builder(
        &self,
        client: &reqwest::Client,
        url: reqwest::Url,
    ) -> reqwest::RequestBuilder {
        client
            .get(url)
            .header(self.provider.api_key_header(), &self.api_key)
    }

    pub async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<T, TwitterApiError> {
        let client = reqwest::Client::new();
        let url = self.request_url(endpoint, params.as_ref())?;
        let request = self.request_builder(&client, url);

        let response = request
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn twitter_xquik_request_url_encodes_query_parameters() {
        let client = TwitterApiClient::new_xquik("test".to_string());
        let mut params = HashMap::new();
        params.insert(
            "q".to_string(),
            "AI & markets #solana from:xquikcom".to_string(),
        );
        params.insert("queryType".to_string(), "Latest".to_string());

        let url = client
            .request_url("/x/tweets/search", Some(&params))
            .unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("xquik.com"));
        assert_eq!(url.path(), "/api/v1/x/tweets/search");
        assert_eq!(
            url.query_pairs().find(|(key, _)| key == "q").unwrap().1,
            "AI & markets #solana from:xquikcom"
        );
    }

    #[test]
    fn twitter_xquik_uses_default_response_contract() {
        let client = TwitterApiClient::new_xquik("test".to_string());
        let request = client
            .request_builder(
                &reqwest::Client::new(),
                reqwest::Url::parse(
                    "https://xquik.com/api/v1/x/tweets/search",
                )
                .unwrap(),
            )
            .build()
            .unwrap();

        assert_eq!(request.headers()["x-api-key"], "test");
        assert!(!request.headers().contains_key("xquik-api-contract"));
    }

    #[test]
    fn twitter_xquik_provider_is_inferred_from_base_url() {
        let client = TwitterApiClient::new(
            "test".to_string(),
            Some("https://xquik.com/api/v1".to_string()),
        );
        let lookalike = TwitterApiClient::new(
            "test".to_string(),
            Some("https://xquik.com.example/api/v1".to_string()),
        );

        assert_eq!(client.provider(), TwitterApiProvider::Xquik);
        assert_eq!(lookalike.provider(), TwitterApiProvider::TwitterApiIo);
    }

    #[test]
    fn twitter_xquik_user_identifier_rejects_path_segments() {
        assert_eq!(xquik_user_identifier("@xquikcom").unwrap(), "xquikcom");
        assert!(xquik_user_identifier("../tweets").is_err());
    }
}
