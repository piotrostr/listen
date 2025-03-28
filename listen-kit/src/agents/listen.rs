use std::sync::Arc;
use tokio::sync::RwLock;

use super::on_chain_analytics::create_on_chain_analytics_agent;
use crate::{
    common::{
        gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
    },
    reasoning_loop::{Model, ReasoningLoop, StreamResponse},
    signer::SignerContext,
    solana::util::make_test_signer,
};
use anyhow::Result;
use rig::streaming::StreamingPrompt;
use rig_tool_macro::tool;

// maybe just put out all of the agents in the bridge and stream the outputs using channels, display this on the frontend based on { agent: { output: ..events } }?
// could be simple and deadly effective
pub struct ListenBridge {}

// Listen, as the swarm leader, plans out the task which is delegated to subsequent agents
// it then can assess the outputs and evaluate as done or needs more information, or a retry

pub async fn create_listen_agent() -> Result<GeminiAgent> {
    let agent = gemini_agent_builder()
        .tool(DelegateToOnChainAnalytics)
        .build();
    Ok(agent)
}

// TODO possibly pass in a summary of the conversation, can be done once and re-used on per-agent pass
#[tool(
    description = "Delegate a task to on-chain analytics agent, provide a prompt that encapsulates the problem"
)]
pub async fn delegate_to_on_chain_analytics(
    prompt: String,
) -> Result<String> {
    let agent = create_on_chain_analytics_agent().await?;
    // let signer_context = SignerContext::current().await;
    let reasoning_loop =
        ReasoningLoop::new(Model::Gemini(Arc::new(agent))).with_stdout(false);
    // TODO broadcase the response here onto the bridge, potentially with websocket streams
    // or sth, redis queue could work too but it'd need a broadcaster still
    // could be potentially on the main rx channel, but could get messy
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let res = Arc::new(RwLock::new(String::new()));

    let res_ptr = res.clone();

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            match response {
                StreamResponse::Message(message) => {
                    println!("{}", message);
                    res_ptr.write().await.push_str(&message);
                }
                StreamResponse::ToolResult { id, name, result } => {
                    println!("{} {} {}", name, id, result);
                    res_ptr.write().await.push_str(&result);
                }
                StreamResponse::ToolCall { id, name, params } => {
                    println!("{} {} {}", id, name, params);
                    res_ptr.write().await.push_str(&params);
                }
                _ => {}
            }
        }
    });

    wrap_unsafe(move || async move {
        reasoning_loop.stream(prompt, vec![], Some(tx)).await
    })
    .await?;

    let result = res.read().await.to_string();
    Ok(result)
}

// #[tool(description = "Delegate a task to solana trader agent")]
// pub async fn delegate_to_solana_trader(
//     #[tool_input(description = "The task to delegate")] task: String,
// ) -> Result<String> {
//     let agent = create_solana_trader_agent().await?;
// }
