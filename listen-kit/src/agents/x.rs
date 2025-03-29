use crate::agents::delegate::delegate_to_agent;
use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::twitter_tools::{
    FetchXPost, ResearchXProfile, SearchTweets,
};
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
    let signer = SignerContext::current().await;
    delegate_to_agent(
        prompt,
        create_x_agent(),
        "x_agent".to_string(),
        signer,
        false,
    )
    .await
}
