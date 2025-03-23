pub struct ExaClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ExaClientError {
    #[error("[ExaClient] Missing env: EXA_API_KEY")]
    MissingEnvironmentVariable,
    #[error("[ExaClient] Init Client Error: {0}")]
    InitClientError(#[from] reqwest::Error),
    #[error("[ExaClient] Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("[ExaClient] Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("[ExaClient] Failed to send request: {0}")]
    RequestError(reqwest::Error),
    #[error("[ExaClient] Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
}

impl ExaClient {
    pub fn from_env() -> Result<Self, ExaClientError> {
        let api_key = std::env::var("EXA_API_KEY")?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer {}",
                api_key
            ))?,
        );
        Ok(Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
            base_url: "https://api.exa.ai".to_string(),
        })
    }

    pub async fn post(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ExaClientError> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, url))
            .json(&body)
            .send()
            .await
            .map_err(|e| ExaClientError::RequestError(e))?;
        Ok(response.json().await?)
    }
}
