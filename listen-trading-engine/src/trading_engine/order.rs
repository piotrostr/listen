pub struct Pipeline {
    pub user_id: String,
    pub address: String,
}

pub struct Condition {}

pub struct Order {
    pub user_id: String,
    pub address: String,
    pub caip2: String,
    pub condition: Condition,
    pub evm_transaction: Option<serde_json::Value>,
    pub solana_transaction: Option<String>, // base64
}

impl Order {
    pub fn is_solana(&self) -> bool {
        self.caip2.starts_with("solana")
    }
}
