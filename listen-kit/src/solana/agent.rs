use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    BuyPumpFunToken, DeployPumpFunToken, FetchTokenPrice, GetPortfolio,
    GetPublicKey, GetSolBalance, GetSplTokenBalance, PerformJupiterSwap,
    SearchOnDexScreener, SellPumpFunToken, TransferSol, TransferSplToken,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};

pub async fn create_solana_agent() -> Result<Agent<AnthropicCompletionModel>>
{
    Ok(claude_agent_builder()
        .preamble(&format!(
            "{} {}",
            "you are a solana trading agent that can also interact with pump.fun;", 
            PREAMBLE_COMMON
        ))
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
