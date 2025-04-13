use super::LunarCrushApiError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub struct LunarCrushApiClient {
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LunarCrushApiResponseData {
    pub error: u32,
    pub message: String,
}

impl std::fmt::Display for LunarCrushApiResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.error, self.message)
    }
}

pub type LunarCrushApiResponseError = LunarCrushApiResponseData;

impl LunarCrushApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| {
                "https://lunarcrush.com/api4/public".to_string()
            }),
        }
    }

    pub async fn request<T>(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<T, LunarCrushApiError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(LunarCrushApiError::RequestError)?;

        let mut request_builder = client.get(&url);

        request_builder = request_builder
            .header("Authorization", format!("Bearer {}", self.api_key));

        if let Some(query_params) = params {
            request_builder = request_builder.query(&query_params);
        }

        let response = request_builder
            .send()
            .await
            .map_err(LunarCrushApiError::RequestError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(LunarCrushApiError::RequestError)?;
            tracing::error!(
                "[LunarCrush] API error: Status {} - {}",
                status,
                error_text
            );

            if let Ok(error_data) =
                serde_json::from_str::<LunarCrushApiResponseData>(&error_text)
            {
                return Err(LunarCrushApiError::ApiError(error_data));
            }

            return Err(LunarCrushApiError::InvalidInput(anyhow::anyhow!(
                "API error: status code {} - {}",
                status,
                error_text
            )));
        }

        let body_text = response
            .text()
            .await
            .map_err(LunarCrushApiError::RequestError)?;

        if body_text.trim().is_empty() {
            tracing::error!(
                "[LunarCrush] API has returned an empty response"
            );
            return Err(LunarCrushApiError::InvalidInput(anyhow::anyhow!(
                "API response is empty"
            )));
        }

        tracing::debug!("LunarCrush API Response: {}", body_text);

        match serde_json::from_str::<T>(&body_text) {
            Ok(data) => Ok(data),
            Err(e) => {
                tracing::error!(
                    "[LunarCrush] Error deserializing JSON: {} - Body: {}",
                    e,
                    body_text
                );

                if let Ok(_) =
                    serde_json::from_str::<serde_json::Value>(&body_text)
                {
                    Err(LunarCrushApiError::DeserializeError(e, body_text))
                } else {
                    Err(LunarCrushApiError::DeserializeError(e, body_text))
                }
            }
        }
    }
}
