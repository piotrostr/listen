use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    DeployPumpFunToken, GetPublicKey, GetQuote, GetSolBalance,
    GetSplTokenBalance, Swap,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};
use crate::data::{FetchCandlesticks, FetchTopTokens};
use crate::dexscreener::tools::SearchOnDexScreener;

pub async fn create_solana_agent(
    preamble: Option<String>,
) -> Result<Agent<AnthropicCompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        "you are a solana trading agent that can also interact with pump.fun;",
        PREAMBLE_COMMON
    ));
    Ok(claude_agent_builder()
        .preamble(&preamble)
        .max_tokens(1024)
        .tool(GetQuote)
        .tool(Swap)
        .tool(GetPublicKey)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(DeployPumpFunToken)
        .tool(FetchCandlesticks)
        .tool(FetchTopTokens)
        .build())
}
