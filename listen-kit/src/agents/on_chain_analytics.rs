use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::key_information::extract_key_information;
use crate::common::{
    gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
};
use crate::data::{FetchTokenMetadata, FetchTokenPrice};
use crate::reasoning_loop::{Model, ReasoningLoop, StreamResponse};
use crate::signer::SignerContext;
use crate::solana::tools::{AnalyzeRisk, GetSolBalance, GetSplTokenBalance};
use anyhow::Result;
use rig_tool_macro::tool;

pub fn create_on_chain_analytics_agent() -> GeminiAgent {
    gemini_agent_builder().preamble("You are a deep on-chain research agent. Your goal is to perform thorough recursive analysis:
    1. For each tool call result, analyze if there are more leads to explore
    2. If you find new addresses, tokens, or entities, investigate them
    3. Build a comprehensive picture by following all relevant leads
    4. Don't stop at surface-level information - dig deeper into each finding
    5. If you find something interesting, use other tools to verify and expand on it")
        .tool(FetchTokenMetadata)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(FetchTokenPrice)
        .tool(AnalyzeRisk)
        .build()
}

// TODO possibly pass in a summary of the conversation, can be done once and re-used on per-agent pass
#[tool(description = "Delegate a task to on-chain analytics agent")]
pub async fn delegate_to_on_chain_analytics(
    prompt: String,
) -> Result<String> {
    let reasoning_loop = ReasoningLoop::new(Model::Gemini(Arc::new(
        create_on_chain_analytics_agent(),
    )))
    .with_stdout(false);
    // TODO broadcase the response here onto the bridge, potentially with websocket streams
    // or sth, redis queue could work too but it'd need a broadcaster still
    // could be potentially on the main rx channel, but could get messy
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

    let result = res.read().await.to_string();

    wrap_unsafe(move || async move { extract_key_information(result).await })
        .await
}
