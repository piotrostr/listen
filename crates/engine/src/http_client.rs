//! json in/out, automatic retries
use log::{info, warn};
use serde::Serialize;
use serde_json::Value;

use listen_util::env;

pub struct HttpClient {
    client: reqwest::Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> HttpClient {
        HttpClient {
            client: reqwest::Client::new(),
        }
    }

    pub async fn buy<T: Serialize + ?Sized>(
        &self,
        buy_request: &T,
    ) -> Result<(), reqwest::Error> {
        let url = env("BUYER_URL") + "/buy";
        self._post(&url, buy_request).await
    }

    pub async fn checks<T: Serialize + ?Sized>(
        &self,
        checks_request: &T,
    ) -> Result<(), reqwest::Error> {
        let url = env("CHECKER_URL") + "/checks";
        self._post(&url, checks_request).await
    }

    pub async fn sell<T: Serialize + ?Sized>(
        &self,
        sell_request: &T,
    ) -> Result<(), reqwest::Error> {
        let url = env("SELLER_URL") + "/sell";
        self._post(&url, sell_request).await
    }

    async fn _post<T: Serialize + ?Sized>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<(), reqwest::Error> {
        let mut backoff = 1;
        for _ in 0..5 {
            match self.client.post(url).json(&payload).send().await {
                Ok(response) => {
                    info!(
                        "{} response: {}",
                        url,
                        serde_json::to_string_pretty(
                            &response
                                .json::<Value>()
                                .await
                                .expect("parse json")
                        )
                        .expect("pretty response")
                    );
                    break;
                }
                Err(e) => {
                    warn!("{} error, backing off: {}", url, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        backoff,
                    ))
                    .await;
                    backoff *= 2;
                }
            }
        }

        Ok(())
    }
}
