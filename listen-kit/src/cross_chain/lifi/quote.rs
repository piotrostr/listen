use anyhow::{anyhow, Result};
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

impl QuoteResponse {
    pub fn summary(&self) -> serde_json::Value {
        let estimate = &self.estimate;
        let action = &self.action;

        // Calculate total gas costs in USD
        let total_gas_usd: f64 = estimate
            .gas_costs
            .as_ref()
            .map(|costs| {
                costs
                    .iter()
                    .filter_map(|cost| {
                        cost.amount_usd
                            .as_ref()
                            .and_then(|amount| amount.parse::<f64>().ok())
                    })
                    .sum()
            })
            .unwrap_or(0.0);

        // Calculate total fee costs in USD
        let total_fees_usd: f64 = estimate
            .fee_costs
            .as_ref()
            .map(|costs| {
                costs
                    .iter()
                    .filter_map(|cost| {
                        cost.amount_usd
                            .as_ref()
                            .and_then(|amount| amount.parse::<f64>().ok())
                    })
                    .sum()
            })
            .unwrap_or(0.0);

        serde_json::json!({
            "from": {
                "token": action.from_token.symbol,
                "amount": estimate.from_amount,
                "amount_usd": estimate.from_amount_usd,
                "chain_id": action.from_chain_id,
            },
            "to": {
                "token": action.to_token.symbol,
                "amount": estimate.to_amount,
                "amount_min": estimate.to_amount_min,
                "amount_usd": estimate.to_amount_usd,
                "chain_id": action.to_chain_id,
            },
            "costs": {
                "gas_usd": total_gas_usd,
                "fees_usd": total_fees_usd,
                "total_usd": total_gas_usd + total_fees_usd
            },
            "execution_time_seconds": estimate.execution_duration,
            "slippage_percent": action.slippage.unwrap_or(0.0),
            "transaction_request": self.transaction_request.as_ref().map(|r| r.is_evm().then(|| r.to_json_rpc().unwrap())),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub data: String,
    // EVM specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl TransactionRequest {
    /// Returns true if this is an EVM transaction request
    pub fn is_evm(&self) -> bool {
        self.chain_id.is_some()
    }

    /// Returns true if this is a Solana transaction request
    pub fn is_solana(&self) -> bool {
        !self.is_evm()
    }

    /// Converts the transaction request to a JSON-RPC compatible format
    /// Returns None for Solana transactions
    pub fn to_json_rpc(&self) -> Result<serde_json::Value> {
        if !self.is_evm() {
            return Err(anyhow!("Not an EVM transaction"));
        }

        // For EVM transactions, construct JSON-RPC format
        Ok(serde_json::json!({
            "from": self.from,
            "to": self.to,
            "data": self.data,
            "chainId": self.chain_id,
            "gas": self.gas_limit.as_ref().map(|s| format!("0x{}", s.trim_start_matches("0x"))),
            "gasPrice": self.gas_price.as_ref().map(|s| format!("0x{}", s.trim_start_matches("0x"))),
            "value": self.value.as_ref().map(|s| format!("0x{}", s.trim_start_matches("0x"))),
        }))
    }
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
