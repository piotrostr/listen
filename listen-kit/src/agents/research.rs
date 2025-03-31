use std::sync::Arc;

use anyhow::Result;
use privy::util::base64encode;
use rig::{
    completion::Prompt,
    message::{ContentFormat, Image, ImageMediaType},
};
use rig_tool_macro::tool;

use crate::{
    agents::delegate::delegate_to_agent,
    common::{gemini_agent_builder, wrap_unsafe, GeminiAgent},
    data::twitter_tools::{FetchXPost, ResearchXProfile, SearchTweets},
    data::{AnalyzePageContent, SearchWeb},
    reasoning_loop::Model,
    signer::SignerContext,
};

pub fn create_research_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are a comprehensive research agent. Your goal is to perform thorough recursive analysis across web and social media:
        1. For each tool call result, analyze if there are more leads to explore
        2. If you find new pages, profiles, posts, or topics, investigate them
        3. Build a comprehensive picture by following all relevant leads
        4. Don't stop at surface-level information - dig deeper into each finding
        5. If you find something interesting, use other tools to verify and expand on it")
        .tool(ViewImage)
        .tool(SearchWeb)
        .tool(AnalyzePageContent)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .build()
}

#[tool(
    description = "Delegate a task to research agent. It can search the web, analyze pages, view images, and research X (Twitter)"
)]
pub async fn delegate_to_research_agent(prompt: String) -> Result<String> {
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_research_agent())),
        "research_agent".to_string(),
        SignerContext::current().await,
        false,
    )
    .await
}

// TODO delegate with channel pass
#[tool(description = "View an image")]
pub async fn view_image(image_url: String) -> Result<String> {
    let agent = gemini_agent_builder().preamble("You are a helpful assistant that can read images and describe them").build();
    let image_data = reqwest::get(image_url).await?.bytes().await?;
    let data = base64encode(&image_data);
    wrap_unsafe(move || async move {
        agent
            .prompt(Image {
                data,
                format: Some(ContentFormat::Base64),
                media_type: Some(ImageMediaType::PNG), // this doesn't matter for Gemini
                detail: None,
            })
            .await
            .map_err(|e| anyhow::anyhow!("Error viewing image: {}", e))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_image() {
        let res = view_image("https://ipfs.io/ipfs/QmX1UG3uu6dzQaEycNnwea9xRSwZbGPFEdv8XPXJjBUVsT".to_string())
            .await
            .unwrap();
        tracing::info!("{}", res);
    }
}
