use super::tools::{
    ApproveTokenForRouterSpend, GetErc20Balance, GetEthBalance, Trade,
    TransferErc20, TransferEth, VerifySwapRouterHasAllowance, WalletAddress,
};
use crate::common::{claude_agent_builder, ClaudeAgent};

pub fn create_evm_agent(preamble: Option<String>) -> ClaudeAgent {
    let preamble =
        preamble.unwrap_or("you are an ethereum trading agent".to_string());
    claude_agent_builder()
        .preamble(&preamble)
        .tool(Trade)
        .tool(TransferEth)
        .tool(TransferErc20)
        .tool(WalletAddress)
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(ApproveTokenForRouterSpend)
        .tool(VerifySwapRouterHasAllowance)
        .build()
}
