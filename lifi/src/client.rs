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

    #[error("[LiFi] Invalid status code: {0}, error: {1}")]
    InvalidStatusCode(reqwest::StatusCode, serde_json::Value),

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

    /// Extracts a simplified error message from a complex LiFi API error response
    fn extract_error_message(value: &serde_json::Value) -> Option<String> {
        // First try to get the main error message
        if let Some(message) = value.get("message").and_then(|m| m.as_str()) {
            return Some(message.to_string());
        }

        // If there's an errors object with failed array, try to get the first meaningful error
        if let Some(errors) = value.get("errors").and_then(|e| e.as_object()) {
            if let Some(failed) = errors.get("failed").and_then(|f| f.as_array()) {
                for error in failed {
                    // Look for subpaths which often contain the actual error details
                    if let Some(subpaths) = error.get("subpaths").and_then(|s| s.as_object()) {
                        for (_, path_errors) in subpaths {
                            if let Some(error_array) = path_errors.as_array() {
                                for error_detail in error_array {
                                    if let (Some(code), Some(message)) = (
                                        error_detail.get("code").and_then(|c| c.as_str()),
                                        error_detail.get("message").and_then(|m| m.as_str()),
                                    ) {
                                        return Some(format!("{}: {}", code, message));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
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
        tracing::info!(?status, "GET {}, {}", endpoint, status);

        let res: serde_json::Value = response
            .json()
            .await
            .map_err(LiFiClientError::ParseBodyError)?;

        if !status.is_success() {
            let simplified_error =
                Self::extract_error_message(&res).unwrap_or_else(|| "Unknown error".to_string());
            tracing::debug!("Full error response: {}", res);
            tracing::error!(
                ?status,
                "GET {} failed: {}, params: {:?}",
                endpoint,
                simplified_error,
                params
            );
            return Err(LiFiClientError::InvalidStatusCode(
                status,
                simplified_error.into(),
            ));
        }

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

        let res: serde_json::Value = response
            .json()
            .await
            .map_err(LiFiClientError::ParseBodyError)?;

        if !status.is_success() {
            return Err(LiFiClientError::InvalidStatusCode(status, res));
        }
        tracing::debug!(?status, "POST {}", endpoint);

        serde_json::from_value(res).map_err(LiFiClientError::DeserializeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_error_message() {
        // ... existing test cases ...

        // Test with real LiFi API error response
        let real_lifi_error = json!({
            "code": 1002,
            "errors": {
                "failed": [{
                    "overallPath": "42161:USDC~42161:ETH-42161:ETH-mayan-1151111081099710:SOL",
                    "subpaths": {
                        "42161:ETH-mayan-1151111081099710:SOL": [{
                            "code": "AMOUNT_TOO_LOW",
                            "errorType": "NO_QUOTE",
                            "message": "AMOUNT_TOO_LOW",
                            "tool": "mayan"
                        }],
                        "42161:USDC~42161:ETH": [{
                            "code": "TOOL_NOT_ALLOWED",
                            "errorType": "NO_QUOTE",
                            "message": "The tool in this quote is not allowed by LI.FI contracts. Please report this to the LI.FI team.",
                            "tool": "sushiswap"
                        }]
                    }
                }]
            },
            "message": "No available quotes for the requested transfer"
        });

        assert_eq!(
            LiFiClient::extract_error_message(&real_lifi_error),
            Some("No available quotes for the requested transfer".to_string())
        );

        // Test case where top-level message is missing but subpath errors exist
        let real_lifi_error_no_message = json!({
            "code": 1002,
            "errors": {
                "failed": [{
                    "subpaths": {
                        "42161:ETH-mayan-1151111081099710:SOL": [{
                            "code": "AMOUNT_TOO_LOW",
                            "errorType": "NO_QUOTE",
                            "message": "AMOUNT_TOO_LOW",
                            "tool": "mayan"
                        }]
                    }
                }]
            }
        });

        assert_eq!(
            LiFiClient::extract_error_message(&real_lifi_error_no_message),
            Some("AMOUNT_TOO_LOW: AMOUNT_TOO_LOW".to_string())
        );
    }
}
