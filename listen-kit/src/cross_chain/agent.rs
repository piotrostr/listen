use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use crate::{
    common::{claude_agent_builder, PREAMBLE_COMMON},
    cross_chain::tools::{ApproveToken, CheckApproval, GetQuote, Swap},
    dexscreener::tools::SearchOnDexScreener,
};

pub async fn create_cross_chain_agent(
) -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble(&format!(
            "{} {}",
            "you are a cross-chain trading agent", PREAMBLE_COMMON,
        ))
        .tool(SearchOnDexScreener)
        .tool(GetQuote)
        .tool(Swap)
        .tool(ApproveToken)
        .tool(CheckApproval)
        .build())
}
