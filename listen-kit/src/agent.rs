use anyhow::Result;
use rig::agent::Agent;
use rig::providers::openai::CompletionModel;

use crate::tools::{
    initialize, BuyPumpToken, DeployToken, FetchTokenPrice, GetBalance,
    GetTokenBalance, Portfolio, SellPumpToken, Trade, TransferSol,
    TransferToken, WalletAddress,
};
use crate::util::env;

pub async fn create_trader_agent() -> Result<Agent<CompletionModel>> {
    dotenv::dotenv().ok();
    initialize(env("PRIVATE_KEY")).await;

    Ok(rig::providers::openai::Client::from_env()
        .agent(rig::providers::openai::GPT_4O)
        .preamble("you are a solana blockchain agent with tools for trading and managing tokens. if you are missing any inputs, ask the user to fill in the blanks, show confirmation before sending it")
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
        .tool(SellPumpToken)
        .build())
}
