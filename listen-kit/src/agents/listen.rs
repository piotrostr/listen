use rig::completion::Prompt;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::on_chain_analytics::create_on_chain_analytics_agent;
use crate::{
    agents::x::create_x_agent,
    common::{
        gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
    },
    data::FetchTokenMetadata,
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

pub fn create_listen_agent() -> GeminiAgent {
    gemini_agent_builder()
        .tool(DelegateToXAgent)
        .tool(FetchTokenMetadata)
        .preamble(
            r#"You are a planning agent, a coordinator that delegates tasks to specialized agents.
            Your goal is to dig as deep as possible into each topic by:
            1. Breaking down complex queries into smaller, focused questions
            2. Delegating each question to appropriate agents
            3. Analyzing their responses to identify gaps or areas needing deeper investigation
            4. Continuing to delegate follow-up questions until you have comprehensive insights
            
            Always make multiple tool calls to build a complete picture. Never be satisfied with surface-level information.
            For each task, provide a series of prompts that progressively dig deeper into the topic.
            
            Format your investigation plan like this:
            1. Initial question: [delegate to appropriate agent]
            2. Follow-up areas based on response
            3. Deep-dive questions for each area
            
            Keep investigating until you have explored all relevant angles."#,
        )
        .build()
}

pub async fn extract_key_information(output: String) -> Result<String> {
    let agent = gemini_agent_builder().preamble("Extract the key information from the given output. Keep your answer brief").build();
    let res = agent.prompt(output.clone()).await.map_err(|e| {
        anyhow::anyhow!("Error extracting key information: {}", e)
    })?;

    tracing::info!("extract key information input: {}", output);

    tracing::info!("extract key information result: {}", res);

    Ok(res)
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

#[tool(description = "Delegate a task to x agent")]
pub async fn delegate_to_x_agent(prompt: String) -> Result<String> {
    let reasoning_loop =
        ReasoningLoop::new(Model::Gemini(Arc::new(create_x_agent())))
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
