use reqwest::Client;
use serde::{Deserialize, Serialize};

use anyhow::Result;

const BASE_URL: &str = "https://li.quest/v1";

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
    ) -> Result<T> {
        let mut request =
            self.client.get(format!("{}{}", BASE_URL, endpoint));

        if let Some(api_key) = &self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request.query(params).send().await?;
        let status = response.status();
        tracing::info!(?status, "GET {}", endpoint);
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status code {}, {}",
                status,
                response.text().await?
            ));
        }
        let res: serde_json::Value = response.json().await?;
        // TODO remove this later
        tracing::info!("{:#?}", res);

        Ok(serde_json::from_value(res)?)
    }

    #[allow(dead_code)]
    pub async fn post<T: for<'a> Deserialize<'a>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let mut request =
            self.client.post(format!("{}{}", BASE_URL, endpoint));

        if let Some(api_key) = &self.api_key {
            request = request.header("x-lifi-api-key", api_key);
        }

        let response = request.json(body).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status code {}, {}",
                status,
                response.text().await?
            ));
        }
        tracing::info!(?status, "POST {}", endpoint);
        Ok(response.json().await?)
    }
}
