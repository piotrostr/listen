use crate::agents::key_information::extract_key_information;
use crate::common::{
    gemini_agent_builder, spawn_with_signer, wrap_unsafe, GeminiAgent,
};
use crate::data::twitter_tools::{
    FetchXPost, ResearchXProfile, SearchTweets,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::reasoning_loop::{Model, ReasoningLoop, StreamResponse};
use crate::signer::SignerContext;
use anyhow::Result;
use rig_tool_macro::tool;

pub fn create_x_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are a deep X research agent. Your goal is to perform thorough recursive analysis:
        1. For each tool call result, analyze if there are more leads to explore
        2. If you find new profiles, posts, or topics, investigate them
        3. Build a comprehensive picture by following all relevant leads
        4. Don't stop at surface-level information - dig deeper into each finding
        5. If you find something interesting, use other tools to verify and expand on it")
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .build()
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
