use anyhow::Result;
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use rig_tool_macro::tool;

use crate::{hype::parse_evm_address, signer::SignerContext};

#[tool(description = "
Gets the complete orderbook snapshot for a given coin. Example response:
{
  \"coin\": \"ETH\",
  \"levels\": [
    [
      {\"n\": 1, \"px\": \"2545.4\", \"sz\": \"11.7811\"}, // 1 order at 2545.4, size 11.7811 ETH
      {\"n\": 12, \"px\": \"2545.0\", \"sz\": \"136.8789\"}, // 12 orders at 2545.0, size 136.8789 ETH
      {\"n\": 17, \"px\": \"2544.9\", \"sz\": \"144.4251\"}, // 17 orders at 2544.9, size 144.4251 ETH
      // ... more orders deeper on the bid (buy) side, skipped for brevity
    ],
    [
      {\"n\": 1, \"px\": \"2545.5\", \"sz\": \"0.0061\"}, // 1 order at 2545.5, size 0.0061 ETH
      {\"n\": 10, \"px\": \"2545.6\", \"sz\": \"40.0728\"}, // 10 orders at 2545.6, size 40.0728 ETH
      {\"n\": 6, \"px\": \"2545.7\", \"sz\": \"102.1028\"}, // 6 orders at 2545.7, size 102.1028 ETH
      // ... more orders deeper on the ask (sell) side, skipped for brevity
    ]
  ],
  \"time\": 1748279333332
}")]
pub async fn get_l2_snapshot(coin: String) -> Result<serde_json::Value> {
    // thread-local this, possibly onto the signer
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let info = client.l2_snapshot(coin).await?;
    Ok(serde_json::to_value(info)?)
}

#[tool(description = "
Gets the open orders for the current user
")]
pub async fn get_open_orders() -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let address = SignerContext::current().await.address();
    let res = client.open_orders(parse_evm_address(address)?).await?;
    Ok(serde_json::to_value(res)?)
}

#[tool(description = "
Gets the balance overview of the current user. Example response:
")]
pub async fn get_balance_overview() -> Result<serde_json::Value> {
    let address = SignerContext::current().await.address();
    _get_balance_overview(parse_evm_address(address)?).await
}

#[tool(description = "
Gets the latest price for a coin.
")]
pub async fn get_latest_price(coin: String) -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.l2_snapshot(coin).await?;
    let bid = res.levels[0][0].px.parse::<f64>()?;
    let ask = res.levels[1][0].px.parse::<f64>()?;
    Ok(serde_json::json!({
        "bid": bid,
        "ask": ask,
    }))
}

pub async fn _get_balance_overview(
    address: H160,
) -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.user_state(address).await?;
    Ok(serde_json::to_value(res)?)
}

#[cfg(test)]
mod tests {

    use super::*;

    const TEST_ADDRESS: &str = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";

    #[tokio::test]
    async fn test_get_balance_overview() {
        let res = _get_balance_overview(TEST_ADDRESS.parse().unwrap())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }
}
