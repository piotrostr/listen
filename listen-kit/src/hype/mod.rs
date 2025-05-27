use anyhow::Result;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use rig::{agent::AgentBuilder, streaming::StreamingCompletionModel};
use rig_tool_macro::tool;

use crate::agent::Features;
use crate::common::OpenRouterAgent;
use crate::{
    agent::model_to_versioned_model, common::openrouter_agent_builder,
};

const PREAMBLE: &str = "You are a Hyperliquid assistant. Hyperliquid is a
high-performance decentralized derivatives exchange that processes billions of
dollars in daily trading volume. You have access to real-time market data
through specialized tools and aim to help users understand and interact with the
platform effectively. You can provide information about prices, order books, and
market conditions to help users make informed decisions, as well as execute orders on their behalf.";

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

pub fn equip_with_hype_tools<M: StreamingCompletionModel>(
    agent: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent.tool(GetL2Snapshot)
}

pub fn create_hype_agent_openrouter(
    model: Option<String>,
    _features: Features,
    _language: String,
) -> OpenRouterAgent {
    let model = model_to_versioned_model(model.unwrap_or_default());
    let agent = equip_with_hype_tools(openrouter_agent_builder(Some(model)))
        .preamble(PREAMBLE)
        .build();
    agent
}
