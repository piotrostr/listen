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

        // Group costs by token
        let mut token_costs: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Add fee costs
        if let Some(fees) = &estimate.fee_costs {
            for fee in fees {
                if let Some(amount) = &fee.amount {
                    let entry = token_costs
                        .entry(fee.token.symbol.clone())
                        .or_insert_with(|| "0".to_string());
                    // Sum up amounts as strings to handle large numbers
                    *entry = (amount.parse::<u64>().unwrap_or(0)
                        + entry.parse::<u64>().unwrap_or(0))
                    .to_string();
                }
            }
        }

        // Add gas costs
        if let Some(gas_costs) = &estimate.gas_costs {
            for cost in gas_costs {
                let entry = token_costs
                    .entry(cost.token.symbol.clone())
                    .or_insert_with(|| "0".to_string());
                // Sum up amounts as strings to handle large numbers
                *entry = (cost.amount.parse::<u64>().unwrap_or(0)
                    + entry.parse::<u64>().unwrap_or(0))
                .to_string();
            }
        }

        serde_json::json!({
            "from": {
                "token": action.from_token.symbol,
                "address": action.from_token.address,
                "decimals": action.from_token.decimals,
                "amount": estimate.from_amount,
                "chain_id": action.from_chain_id,
            },
            "to": {
                "token": action.to_token.symbol,
                "address": action.to_token.address,
                "decimals": action.to_token.decimals,
                "amount": estimate.to_amount,
                "amount_min": estimate.to_amount_min,
                "chain_id": action.to_chain_id,
            },
            "costs": token_costs,
            "execution_time_seconds": estimate.execution_duration,
            "slippage_percent": action.slippage.unwrap_or(0.0),
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
            "chain_id": self.chain_id,
            "gas": self.gas_limit,
            "gas_price": self.gas_price,
            "value": self.value,
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

#[cfg(test)]
mod tests {
    use super::TransactionRequest;

    #[test]
    fn test_serialize_to_jsonrpc() {
        let tx_request = serde_json::json!({
          "value": "0x0",
          "to": "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE",
          "data": "not relevant",
          "chainId": 56,
          "gasPrice": "0x3b9aca00",
          "gasLimit": "0x55535",
          "from": "0x552008c0f6870c2f77e5cC1d2eb9bdff03e30Ea0"
        });
        let parsed: TransactionRequest = serde_json::from_value(tx_request.clone()).unwrap();
        assert!(parsed.is_evm());

        dbg!(&parsed);

        dbg!(parsed.to_json_rpc().unwrap());
    }
}
