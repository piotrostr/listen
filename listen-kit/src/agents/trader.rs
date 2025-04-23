use crate::{
    agents::delegate::delegate_to_agent,
    common::{gemini_agent_builder, GeminiAgent},
    data::{
        evm_fallback_tools::{
            FetchPriceActionAnalysisEvm, FetchTokenMetadataEvm,
        },
        FetchTokenMetadata, FetchTokenPrice,
    },
    evm::tools::{GetErc20Balance, GetEthBalance},
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

const PREAMBLE_EN: &str = "You are a comprehensive analysis and trading agent. Your goal is to perform thorough research and trading:
        1. Analyze market conditions, liquidity, and trading opportunities
        2. Investigate tokens, addresses, and entities on-chain
        3. Follow interesting leads and dig deeper into findings
        4. Build complete pictures by analyzing on-chain data and market factors
        5. Verify opportunities with token metadata, balances, and quotes
        6. Recommend actions based on comprehensive risk/reward analysis. 
        Always use English.";

const PREAMBLE_ZH: &str =
    "你是一个全面的 分析和交易代理。你的目标是进行彻底的研究和交易：
        1. 分析市场条件、流动性和交易机会
        2. 调查代币、地址和链上实体
        3. 跟随有趣的话题并深入挖掘
        4. 通过分析链上数据和市场因素建立完整图景
        5. 验证机会与代币元数据、余额和报价
        6. 基于全面的风险/回报分析推荐行动. 
        请使用中文";

pub fn create_solana_trader_agent(locale: String) -> GeminiAgent {
    gemini_agent_builder()
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
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
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(FetchTokenMetadataEvm)
        .tool(FetchPriceActionAnalysisEvm)
        .build()
}

#[tool(
    description = "Delegate a task to the trader agent. It can analyze on-chain data, perform swaps, fetch token info, check balances, and schedule advanced orders on both Solana and EVM chains"
)]
pub async fn delegate_to_solana_trader_agent(
    prompt: String,
) -> Result<String> {
    let ctx = SignerContext::current().await;
    let user_id = ctx.user_id().unwrap_or_default();
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_solana_trader_agent(ctx.locale()))),
        "solana_trader_agent".to_string(),
        ctx,
        false,
        user_id,
    )
    .await
}
