use crate::{
    agents::delegate::delegate_to_agent,
    common::{gemini_agent_builder, GeminiAgent},
    data::{FetchTokenMetadata, FetchTokenPrice},
    faster100x::AnalyzeHolderDistribution,
    reasoning_loop::Model,
    signer::SignerContext,
    solana::{
        advanced_orders::CreateAdvancedOrder,
        tools::{
            AnalyzeRisk, DeployPumpFunToken, GetQuote, GetSolBalance,
            GetSplTokenBalance, Swap,
        },
    },
};
use anyhow::Result;
use rig_tool_macro::tool;
use std::sync::Arc;

pub fn create_solana_trader_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are a comprehensive Solana analysis and trading agent. Your goal is to perform thorough research and trading:
        1. Analyze market conditions, liquidity, and trading opportunities
        2. Investigate tokens, addresses, and entities on-chain
        3. Follow interesting leads and dig deeper into findings
        4. Build complete pictures by analyzing on-chain data and market factors
        5. Verify opportunities with token metadata, balances, and quotes
        6. Recommend actions based on comprehensive risk/reward analysis")
        .tool(GetQuote)
        .tool(DeployPumpFunToken)
        .tool(CreateAdvancedOrder)
        .tool(Swap)
        .tool(FetchTokenMetadata)
        .tool(GetSolBalance)
        .tool(AnalyzeHolderDistribution)
        .tool(GetSplTokenBalance)
        .tool(FetchTokenPrice)
        .tool(AnalyzeRisk)
        .build()
}

#[tool(
    description = "Delegate a task to Solana trader agent. It can analyze on-chain data, perform swaps, fetch token info, check balances, and schedule advanced orders"
)]
pub async fn delegate_to_solana_trader_agent(
    prompt: String,
) -> Result<String> {
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_solana_trader_agent())),
        "solana_trader_agent".to_string(),
        SignerContext::current().await,
        false,
    )
    .await
}
