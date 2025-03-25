use anyhow::Result;
use reqwest;
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

use crate::signer::SignerContext;

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub price: f64,
    pub price_asset_mint: String,
    pub condition: String, // "PriceBelow" or "PriceAbove"
    pub user_id: String,
}

#[tool(description = "
Creates an advanced order

Params:
  input_token: the mint of the token to be swapped
  output_token: the mint of the token to be received
  amount: the amount of the input token to be swapped, accounting for decimals
  price: the price of the price_asset_mint, denoted in USD
  price_asset_mint: the mint of the asset of which price is checked by the condition.
  condition: the condition of the order. Can be \"PriceBelow\" or \"PriceAbove\".

returns the ID of the submitted order
")]
pub async fn create_advanced_order(
    input_token: String,
    output_token: String,
    amount: String,
    price: f64,
    price_asset_mint: String,
    condition: String, // "PriceBelow" or "PriceAbove"
) -> Result<String> {
    let user_id = match SignerContext::current().await.user_id() {
        Some(user_id) => user_id,
        None => {
            return Err(anyhow::anyhow!(
                "User ID not found, Privy signer required"
            ))
        }
    };
    let order = Order {
        input_token,
        output_token,
        amount,
        price,
        price_asset_mint,
        condition,
        user_id,
    };

    submit_order_internal(&order).await
}

pub fn list_orders() -> Result<Vec<Order>> {
    Ok(vec![])
}

pub async fn submit_order_internal(order: &Order) -> Result<String> {
    let request = serde_json::json!({
        "user_id": order.user_id,
        "pipeline": {
            "steps": [
                {
                    "action": {
                        "type": "SwapOrder",
                        "input_token": order.input_token,
                        "output_token": order.output_token,
                        "amount": order.amount,
                    },
                    "conditions": [
                        {
                            "type": order.condition,
                            "asset": order.price_asset_mint,
                            "value": order.price,
                        }
                    ]
                }
            ]
        }
    });

    // Create and send the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:6901/internal/create_pipeline") // Adjust URL as needed
        .json(&request) // Send the pipeline JSON directly
        .send()
        .await?;

    let status = response.status();
    match status {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
        _ => {
            let restext = response.text().await?;
            tracing::error!(
                "create_advanced_order response: {}, {}",
                status,
                restext
            );
            return Err(anyhow::anyhow!(
                "create_advanced_order response: {}, {}",
                status,
                restext
            ));
        }
    }

    // Process the response
    let result = response.json::<serde_json::Value>().await?;

    // Extract pipeline ID or relevant information
    let pipeline_id = result["response"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing pipeline ID in response"))?;

    Ok(pipeline_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_order_internal() {
        let order = Order {
            input_token: "So11111111111111111111111111111111111111112"
                .to_string(),
            amount: "40000000".to_string(),
            output_token: "5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp"
                .to_string(),
            price_asset_mint: "So11111111111111111111111111111111111111112"
                .to_string(),
            condition: "PriceBelow".to_string(),
            price: 0.00019,
            user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
        };
        let id = submit_order_internal(&order).await.unwrap();
        println!("{}", id);
    }
}
