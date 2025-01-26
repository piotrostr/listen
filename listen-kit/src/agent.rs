use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::providers::anthropic::completion::CompletionModel;

#[cfg(feature = "solana")]
use crate::solana::{
    tools::{
        initialize, BuyPumpToken, DeployToken, FetchPairInfo,
        FetchTokenPrice, GetBalance, GetTokenBalance, Portfolio, Scan,
        SellPumpToken, Trade, TransferSol, TransferToken, WalletAddress,
    },
    util::env,
};

pub fn claude_agent_builder() -> AgentBuilder<CompletionModel> {
    rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
}

pub async fn default_agent() -> Result<Agent<CompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("be nice to the users")
        .max_tokens(1024)
        .build())
}

#[cfg(feature = "solana")]
pub async fn create_trader_agent() -> Result<Agent<CompletionModel>> {
    initialize(env("SOLANA_PRIVATE_KEY")).await;

    Ok(claude_agent_builder()
        .preamble("you are a solana trading agent")
        .max_tokens(1024)
        .tool(Trade)
        .tool(TransferSol)
        .tool(TransferToken)
        .tool(WalletAddress)
        .tool(GetBalance)
        .tool(GetTokenBalance)
        .tool(DeployToken)
        .tool(FetchTokenPrice)
        .tool(BuyPumpToken)
        .tool(Portfolio)
        .tool(FetchPairInfo)
        .tool(SellPumpToken)
        .build())
}

#[cfg(feature = "solana")]
pub async fn create_data_agent() -> Result<Agent<CompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("you are a solana blockchain agent with tools for fetching data and information about tokens and pairs")
        .max_tokens(1024)
        .tool(FetchPairInfo)
        .build())
}

#[cfg(feature = "solana")]
pub async fn create_scanner_agent() -> Result<Agent<CompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("you are a screening agent, you are screening newly listed tokens, your purpose is to validate whether a given token is real or not by checking if its contract address is present in the social links")
        .tool(Scan)
        .build())
}
