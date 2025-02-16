use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOrder {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
}
