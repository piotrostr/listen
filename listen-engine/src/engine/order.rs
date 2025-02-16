use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOrder {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyOrder {
    pub user_id: String,
    pub address: String,
    pub caip2: String,
    pub evm_transaction: Option<serde_json::Value>,
    pub solana_transaction: Option<String>, // base64
}

impl PrivyOrder {
    pub fn is_solana(&self) -> bool {
        self.caip2.starts_with("solana")
    }
}
