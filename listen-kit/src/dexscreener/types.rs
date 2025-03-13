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
    // pub base: Option<f64>,
    // pub quote: Option<f64>,
    // not relevant for us, save tokens
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Volume {
    #[serde(default)]
    pub h24: Option<f64>,
    // #[serde(default)]
    // pub h6: Option<f64>,
    // #[serde(default)]
    // pub h1: Option<f64>,
    // #[serde(default)]
    // pub m5: Option<f64>,
    // not relevant for us, save tokens
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
