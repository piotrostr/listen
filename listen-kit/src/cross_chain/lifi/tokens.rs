use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub address: String,
    pub decimals: Number,
    pub symbol: String,
    pub chain_id: Number,
    pub coin_key: Option<String>,
    pub name: String,
    pub logo_uri: Option<String>,
    pub price_usd: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokensResponse {
    pub tokens: HashMap<String, Vec<Token>>,
}
