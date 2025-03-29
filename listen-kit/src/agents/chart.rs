use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    agents::key_information::extract_key_information,
    common::{
        gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
    },
    data::listen_api_tools::FetchPriceActionAnalysis,
    reasoning_loop::{Model, ReasoningLoop, StreamResponse},
    signer::SignerContext,
};
use anyhow::Result;
use rig_tool_macro::tool;

pub fn create_chart_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are a deep chart analysis agent. Your goal is to perform thorough technical analysis:
        1. For each price action analysis, look for significant patterns and signals
        2. If you find interesting price movements, investigate the timeframes around them
        3. Build a comprehensive picture by analyzing multiple technical indicators
        4. Don't stop at surface-level patterns - dig deeper into each finding
        5. If you find something interesting, verify it against other timeframes and indicators")
        .tool(FetchPriceActionAnalysis)
        .build()
}

#[tool(description = "Delegate a task to chart analysis agent")]
pub async fn delegate_to_chart_agent(prompt: String) -> Result<String> {
    let reasoning_loop =
        ReasoningLoop::new(Model::Gemini(Arc::new(create_chart_agent())))
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
