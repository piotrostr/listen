use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub coin_price: f64,
    pub coin_mint: String,
    pub pc_mint: String,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceUpdate {
    pub pubkey: String,
    pub price: f64,
    pub market_cap: Option<f64>,
    pub timestamp: i64,
}
