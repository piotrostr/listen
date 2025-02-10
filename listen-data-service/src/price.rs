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
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: Option<f64>,
    pub timestamp: i64,
    pub slot: u64,
    pub swap_amount: f64, // denoted as usd
    pub owner: String,
    pub signature: String,
    pub base_in: bool,
}
