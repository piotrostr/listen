use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    pub data: HashMap<String, PriceData>,
    pub time_taken: f64,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PriceData {
    pub id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub price: f64,
}

pub async fn fetch_token_price(mint: String, client: &Client) -> Result<f64> {
    let url = format!("https://api.jup.ag/price/v2?ids={}", mint);
    let res = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await?;
    let data = res.json::<PriceResponse>().await?;
    tracing::debug!(?data, "fetch_token_price");
    Ok(data.data.get(&mint).unwrap().price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_fetch_token_price() {
        let res = fetch_token_price(
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump".to_string(),
            &reqwest::Client::new(),
        )
        .await;
        tracing::debug!(?res, "test_fetch_token_price");
        assert!(res.is_ok());
    }
}
