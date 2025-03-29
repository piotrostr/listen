use rig::completion::Prompt;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::on_chain_analytics::create_on_chain_analytics_agent;
use crate::{
    agents::x::create_x_agent,
    common::{
        claude_agent_builder, gemini_agent_builder, spawn_with_signer,
        wrap_unsafe, ClaudeAgent,
    },
    reasoning_loop::{Model, ReasoningLoop, StreamResponse},
    signer::SignerContext,
};
use anyhow::Result;
use rig_tool_macro::tool;

// maybe just put out all of the agents in the bridge and stream the outputs using channels, display this on the frontend based on { agent: { output: ..events } }?
// could be simple and deadly effective
pub struct ListenBridge {}

// Listen, as the swarm leader, plans out the task which is delegated to subsequent agents
// it then can assess the outputs and evaluate as done or needs more information, or a retry

pub fn create_listen_agent() -> ClaudeAgent {
    claude_agent_builder()
        .tool(DelegateToOnChainAnalytics)
        .tool(DelegateToXAgent)
        .preamble(
            r#"Use your agents to perform deep research, for each task, provide
            a prompt that encapsulates the problem."#,
        )
        .build()
}

pub async fn extract_key_information(output: String) -> Result<String> {
    let agent = gemini_agent_builder().preamble("Extract the key information from the given output. Keep your answer brief").build();
    agent.prompt(output).await.map_err(|e| {
        anyhow::anyhow!("Error extracting key information: {}", e)
    })
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

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let s = response.stringify();
            res_ptr.write().await.push_str(&s);
        }
    });

    wrap_unsafe(move || async move {
        reasoning_loop.stream(prompt, vec![], Some(tx)).await
    })
    .await?;

    let result = res.read().await.to_string();

    wrap_unsafe(move || async move { extract_key_information(result).await })
        .await
}

#[tool(description = "Delegate a task to x agent")]
pub async fn delegate_to_x_agent(prompt: String) -> Result<String> {
    let reasoning_loop =
        ReasoningLoop::new(Model::Gemini(Arc::new(create_x_agent())))
            .with_stdout(false);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamResponse>(1024);
    let res = Arc::new(RwLock::new(String::new()));

    let res_ptr = res.clone();

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let s = response.stringify();
            res_ptr.write().await.push_str(&s);
        }
    });

    let signer = SignerContext::current().await;
    let _ = spawn_with_signer(signer, || async move {
        reasoning_loop.stream(prompt, vec![], Some(tx)).await
    })
    .await;

    let response = res.read().await.to_string();

    wrap_unsafe(
        move || async move { extract_key_information(response).await },
    )
    .await
}
