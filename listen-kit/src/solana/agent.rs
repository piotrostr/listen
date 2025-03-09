use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    DeployPumpFunToken, GetQuote, GetSolBalance, GetSplTokenBalance, Swap,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};
use crate::data::{
    FetchCandlesticks, FetchTokenMetadata, FetchTopTokens, FetchXPost,
    ResearchXProfile,
};
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
        .tool(GetQuote)
        .tool(Swap)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchCandlesticks)
        .tool(FetchTopTokens)
        .tool(DeployPumpFunToken)
        .tool(FetchTokenMetadata)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .build())
}
