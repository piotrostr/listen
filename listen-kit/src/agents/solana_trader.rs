use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    agents::key_information::extract_key_information,
    common::{
        gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
    },
    reasoning_loop::{Model, ReasoningLoop, StreamResponse},
    signer::SignerContext,
    solana::{
        advanced_orders::CreateAdvancedOrder,
        tools::{DeployPumpFunToken, GetQuote, Swap},
    },
};
use anyhow::Result;
use rig_tool_macro::tool;

pub fn create_solana_trader_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are a deep Solana trading analysis agent. Your goal is to perform thorough trading analysis:
        1. For each trading opportunity, analyze market conditions and liquidity
        2. If you find interesting trading setups, investigate the risk/reward
        3. Build a comprehensive picture by analyzing multiple market factors
        4. Don't stop at surface-level analysis - dig deeper into each opportunity
        5. If you find something promising, verify it with quotes and market depth")
        .tool(GetQuote)
        .tool(DeployPumpFunToken)
        .tool(CreateAdvancedOrder)
        .tool(Swap)
        .build()
}

#[tool(
    description = "Delegate a task to Solana trading agent. It can perform swaps and schedule advanced orders"
)]
pub async fn delegate_to_solana_trader_agent(
    prompt: String,
) -> Result<String> {
    let reasoning_loop = ReasoningLoop::new(Model::Gemini(Arc::new(
        create_solana_trader_agent(),
    )))
    .with_stdout(false);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamResponse>(1024);
    let res = Arc::new(RwLock::new(String::new()));

    let res_ptr = res.clone();

    let reader_handle = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let s = response.stringify();
            res_ptr.write().await.push_str(&s);
            if matches!(response, StreamResponse::Message(_)) {
                print!("{}", s);
            }
        }
    });

    let signer = SignerContext::current().await;
    let loop_handle = spawn_with_signer(signer, || async move {
        reasoning_loop.stream(prompt, vec![], Some(tx)).await
    })
    .await;

    let _ = tokio::try_join!(reader_handle, loop_handle);

    let response = res.read().await.to_string();

    wrap_unsafe(
        move || async move { extract_key_information(response).await },
    )
    .await
}
