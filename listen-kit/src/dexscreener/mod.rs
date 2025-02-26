pub mod tools;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexScreenerResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub pairs: Vec<PairInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairInfo {
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "dexId")]
    pub dex_id: String,
    pub url: String,
    #[serde(rename = "pairAddress")]
    pub pair_address: String,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "baseToken")]
    pub base_token: Token,
    #[serde(rename = "quoteToken")]
    pub quote_token: Token,
    #[serde(rename = "priceNative")]
    pub price_native: String,
    #[serde(rename = "priceUsd")]
    pub price_usd: Option<String>,
    pub liquidity: Option<Liquidity>,
    pub volume: Option<Volume>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Liquidity {
    pub usd: Option<f64>,
    pub base: Option<f64>,
    pub quote: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Volume {
    #[serde(default)]
    pub h24: Option<f64>,
    #[serde(default)]
    pub h6: Option<f64>,
    #[serde(default)]
    pub h1: Option<f64>,
    #[serde(default)]
    pub m5: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

pub struct TickerResponse {
    pub mint: String,
}

pub async fn search_ticker(ticker: String) -> Result<DexScreenerResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.dexscreener.com/latest/dex/search/?q={}&limit=8",
        ticker
    );

    let response = client.get(&url).send().await?;

    if response.status().is_client_error() {
        let res = response.text().await?;
        tracing::error!("Error: {:?}", res);
        return Err(anyhow::anyhow!("Error: {:?}", res));
    }

    let data: serde_json::Value = response.json().await?;

    let mut dex_response: DexScreenerResponse = serde_json::from_value(data)?;

    // trim up to 8
    dex_response.pairs.truncate(8);

    Ok(dex_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_ticker_base() {
        let response = search_ticker("brett".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_search_ticker() {
        let response = search_ticker("BONK".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_search_by_mint() {
        tracing::debug!("search_by_mint");
        let response = search_ticker(
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump".to_string(),
        )
        .await
        .unwrap();
        tracing::debug!(?response, "search_by_mint");
        assert_eq!(response.schema_version, "1.0.0");
    }
}
