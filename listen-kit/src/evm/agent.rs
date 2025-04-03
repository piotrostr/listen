use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    ApproveTokenForRouterSpend, GetErc20Balance, GetEthBalance, Trade,
    TransferErc20, TransferEth, VerifySwapRouterHasAllowance, WalletAddress,
};
use crate::common::claude_agent_builder;

pub async fn create_evm_agent(
    preamble: Option<String>,
) -> Result<Agent<AnthropicCompletionModel>> {
    let preamble =
        preamble.unwrap_or("you are an ethereum trading agent".to_string());
    Ok(claude_agent_builder()
        .preamble(&preamble)
        .tool(Trade)
        .tool(TransferEth)
        .tool(TransferErc20)
        .tool(WalletAddress)
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(ApproveTokenForRouterSpend)
        .tool(VerifySwapRouterHasAllowance)
        .build())
}
