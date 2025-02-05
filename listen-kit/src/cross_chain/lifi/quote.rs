use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[allow(dead_code)]
pub enum Order {
    Fastest,
    Cheapest,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Order::Fastest => write!(f, "FASTEST"),
            Order::Cheapest => write!(f, "CHEAPEST"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    pub tool: String,
    pub tool_details: ToolDetails,
    pub action: Action,
    pub estimate: Estimate,
    pub data: Option<Value>,
    pub integrator: Option<String>,
    pub included_steps: Option<Vec<IncludedStep>>,
    pub execution: Option<String>,
    pub transaction_request: Option<TransactionRequest>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionRequest {
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolDetails {
    pub key: String,
    pub name: String,
    pub logo_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub from_chain_id: Number,
    pub from_amount: String,
    pub from_token: Token,
    pub to_chain_id: Number,
    pub to_token: Token,
    pub slippage: Option<f64>,
    pub from_address: String,
    pub to_address: Option<String>,
}

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
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub tool: String,
    pub from_amount: String,
    pub from_amount_usd: Option<String>,
    pub to_amount: String,
    pub to_amount_min: String,
    pub to_amount_usd: Option<String>,
    pub approval_address: String,
    pub fee_costs: Option<Vec<FeeCost>>,
    pub gas_costs: Option<Vec<GasCost>>,
    pub execution_duration: Number,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeCost {
    pub name: String,
    pub description: Option<String>,
    pub percentage: String,
    pub token: Token,
    pub amount: Option<String>,
    pub amount_usd: Option<String>,
    pub included: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GasCost {
    #[serde(rename = "type")]
    pub gas_type: String,
    pub price: Option<String>,
    pub estimate: Option<String>,
    pub limit: Option<String>,
    pub amount: String,
    pub amount_usd: Option<String>,
    pub token: Token,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IncludedStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    pub tool: String,
    pub tool_details: ToolDetails,
    pub action: Action,
    pub estimate: Estimate,
    pub data: Option<Value>,
}
