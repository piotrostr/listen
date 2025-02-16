use reqwest::Client;
use serde::{Deserialize, Serialize};

use anyhow::Result;

const BASE_URL: &str = "https://li.quest/v1";

#[derive(Debug, thiserror::Error)]
pub enum LiFiClientError {
    #[error("[LiFi] Request failed: {0}")]
    RequestFailed(reqwest::Error),

    #[error("[LiFi] Invalid response: {0}")]
    InvalidResponse(serde_json::Error),

    #[error("[LiFi] Parse body error: {0}")]
    ParseBodyError(reqwest::Error),

    #[error("[LiFi] Invalid status code: {0}")]
    InvalidStatusCode(reqwest::StatusCode),

    #[error("[LiFi] Deserialize error: {0}")]
    DeserializeError(serde_json::Error),
}

pub struct LiFiClient {
    client: Client,
    api_key: Option<String>,
}

impl LiFiClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get<T: for<'a> Deserialize<'a>>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T, LiFiClientError> {
        let mut request = self.client.get(format!("{}{}", BASE_URL, endpoint));

        if let Some(api_key) = &self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request
            .query(params)
            .send()
            .await
            .map_err(LiFiClientError::RequestFailed)?;
        let status = response.status();
        tracing::debug!(?status, "GET {}", endpoint);
        if !status.is_success() {
            return Err(LiFiClientError::InvalidStatusCode(status));
        }
        let res: serde_json::Value = response
            .json()
            .await
            .map_err(LiFiClientError::ParseBodyError)?;

        serde_json::from_value(res).map_err(LiFiClientError::DeserializeError)
    }

    #[allow(dead_code)]
    pub async fn post<T: for<'a> Deserialize<'a>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, LiFiClientError> {
        let mut request = self.client.post(format!("{}{}", BASE_URL, endpoint));

        if let Some(api_key) = &self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(LiFiClientError::RequestFailed)?;
        let status = response.status();
        if !status.is_success() {
            return Err(LiFiClientError::InvalidStatusCode(status));
        }
        tracing::debug!(?status, "POST {}", endpoint);
        let res: serde_json::Value = response
            .json()
            .await
            .map_err(LiFiClientError::ParseBodyError)?;

        serde_json::from_value(res).map_err(LiFiClientError::DeserializeError)
    }
}
