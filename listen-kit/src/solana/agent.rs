use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    BuyPumpFunToken, DeployPumpFunToken, FetchTokenPrice, GetPortfolio,
    GetPublicKey, GetSolBalance, GetSplTokenBalance, PerformJupiterSwap,
    SellPumpFunToken, TransferSol, TransferSplToken,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};
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
        .tool(PerformJupiterSwap)
        .tool(TransferSol)
        .tool(TransferSplToken)
        .tool(GetPublicKey)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(FetchTokenPrice)
        .tool(GetPortfolio)
        .tool(SearchOnDexScreener)
        .tool(DeployPumpFunToken)
        .tool(BuyPumpFunToken)
        .tool(SellPumpFunToken)
        .build())
}
