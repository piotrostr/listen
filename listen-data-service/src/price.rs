use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub coin_price: f64,
    pub coin_mint: String,
    pub pc_mint: String,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
}

impl Price {
    pub fn normalized_coin_price(&self) -> f64 {
        let decimal_adjustment =
            10_f64.powi((self.pc_decimals as i32) - (self.coin_decimals as i32));
        self.coin_price / decimal_adjustment
    }

    pub fn market_cap(&self, supply: u64) -> f64 {
        let decimal_adjustment = 10_f64.powi(-(self.coin_decimals as i32));
        self.coin_price * (supply as f64 * decimal_adjustment)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceUpdate {
    pub pubkey: String,
    pub price: f64,
    pub market_cap: Option<f64>,
    pub timestamp: i64,
}
