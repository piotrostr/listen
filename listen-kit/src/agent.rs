use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;

use crate::tools::{
    initialize, BuyPumpToken, DeployToken, FetchPairInfo, FetchTokenPrice,
    GetBalance, GetTokenBalance, Portfolio, SellPumpToken, Trade, TransferSol,
    TransferToken, WalletAddress,
};
use crate::util::env;

pub async fn create_trader_agent() -> Result<Agent<CompletionModel>> {
    dotenv::dotenv().ok();
    initialize(env("PRIVATE_KEY")).await;

    Ok(rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .preamble("you are a solana blockchain agent with tools for trading and managing tokens. if you are missing any inputs, ask the user to fill in the blanks, show confirmation before sending it; in case you get a ticker you dont know, search for it first and ask the user to confirm it is the right one, never assume you know the ticker")
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

pub async fn create_data_agent() -> Result<Agent<CompletionModel>> {
    Ok(rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .preamble("you are a solana blockchain agent with tools for fetching data and information about tokens and pairs")
        .max_tokens(1024)
        .tool(FetchPairInfo)
        .build())
}
