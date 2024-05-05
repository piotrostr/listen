use serde::Deserialize;
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    pub data: HashMap<String, PriceData>,
    pub time_taken: f64,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub id: String,
    pub mint_symbol: String,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
}
