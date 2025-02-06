use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use super::tools::{
    ApproveTokenForRouterSpend, GetErc20Balance, GetEthBalance, Trade,
    TransferErc20, TransferEth, VerifySwapRouterHasAllowance, WalletAddress,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};

pub async fn create_evm_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble(&format!(
            "{} {}",
            "you are an ethereum trading agent", PREAMBLE_COMMON
        ))
        .max_tokens(1024)
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
