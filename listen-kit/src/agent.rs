use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

const PREAMBLE_COMMON: &str = "never assume you know the token address, if you dont have it in the context, search for it";

#[cfg(feature = "evm")]
use crate::evm::tools::{
    ApproveTokenForRouterSpend, GetErc20Balance, GetEthBalance, Trade,
    TransferErc20, TransferEth, VerifySwapRouterHasAllowance, WalletAddress,
};
#[cfg(feature = "solana")]
use crate::solana::tools::{
    BuyPumpFunToken, DeployPumpFunToken, FetchTokenPrice, GetPortfolio,
    GetPublicKey, GetSolBalance, GetSplTokenBalance, PerformJupiterSwap,
    SearchOnDexScreener, SellPumpFunToken, TransferSol, TransferSplToken,
};

pub fn claude_agent_builder() -> AgentBuilder<AnthropicCompletionModel> {
    rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
}

pub async fn plain_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("be nice to the users")
        .max_tokens(1024)
        .build())
}

#[cfg(feature = "solana")]
pub async fn create_solana_agent() -> Result<Agent<AnthropicCompletionModel>>
{
    Ok(claude_agent_builder()
        .preamble(&format!(
            "{} {}",
            "you are a solana trading agent;", PREAMBLE_COMMON
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
        .build())
}

#[cfg(feature = "solana")]
pub async fn create_pump_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble(&format!(
            "{} {}",
            "you are a pump.fun trading agent", PREAMBLE_COMMON,
        ))
        .max_tokens(1024)
        .tool(TransferSol)
        .tool(TransferSplToken)
        .tool(GetPublicKey)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(DeployPumpFunToken)
        .tool(BuyPumpFunToken)
        .tool(SellPumpFunToken)
        .tool(GetPortfolio)
        .build())
}

#[cfg(feature = "evm")]
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
