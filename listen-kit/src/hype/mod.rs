use anyhow::Result;
use ethers::types::H160;
use rig::{agent::AgentBuilder, streaming::StreamingCompletionModel};

use crate::agent::Features;
use crate::common::OpenRouterAgent;
use crate::hype::info::*;
use crate::hype::orders::*;
#[cfg(feature = "hype")]
use crate::{
    agent::model_to_versioned_model, common::openrouter_agent_builder,
};
pub mod info;
pub mod orders;

const PREAMBLE: &str = "You are a Hyperliquid assistant. Hyperliquid is a
high-performance decentralized derivatives exchange that processes billions of
dollars in daily trading volume. You have access to real-time market data
through specialized tools and aim to help users understand and interact with the
platform effectively. You can provide information about prices, order books, and
market conditions to help users make informed decisions, as well as execute orders on their behalf.";

pub fn equip_with_hype_tools<M: StreamingCompletionModel>(
    agent: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent
        .tool(GetL2Snapshot)
        .tool(MarketOpen)
        .tool(GetOpenOrders)
        .tool(DepositUsdc)
        .tool(GetBalanceOverview)
        .tool(GetLatestPrice)
    // .tool(GetPriceLine)
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

pub fn parse_evm_address(address: Option<String>) -> Result<H160> {
    Ok(address
        .ok_or(anyhow::anyhow!("No EVM address found"))?
        .parse::<H160>()?)
}
