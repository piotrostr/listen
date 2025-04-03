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
    data::{
        twitter_tools::{FetchXPost, ResearchXProfile, SearchTweets},
        AnalyzePageContent, SearchWeb,
    },
    lunarcrush::AnalyzeSentiment,
    reasoning_loop::Model,
    signer::SignerContext,
};

const PREAMBLE_EN: &str = "You are a comprehensive research agent. Your goal is to perform thorough recursive analysis across web and social media:
        1. For each tool call result, analyze if there are more leads to explore
        2. If you find new pages, profiles, posts, or topics, investigate them
        3. Build a comprehensive picture by following all relevant leads
        4. Don't stop at surface-level information - dig deeper into each finding
        5. If you find something interesting, use other tools to verify and expand on it";

const PREAMBLE_ZH: &str = "你是一个全面的调研代理。你的目标是进行彻底的递归分析，跨越网络和社交媒体：
        1. 对于每个工具调用结果，分析是否存在更多需要探索的线索
        2. 如果你发现新的页面、个人资料、帖子或话题，调查它们
        3. 通过遵循所有相关线索建立全面图景
        4. 不要停留在表面信息 - 深入挖掘每个发现
        5. 如果你发现有趣的东西，使用其他工具验证并扩展它";

pub fn create_research_agent(locale: String) -> GeminiAgent {
    println!("Creating research agent with locale: {}", locale);
    gemini_agent_builder()
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .tool(ViewImage)
        .tool(SearchWeb)
        .tool(AnalyzePageContent)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(AnalyzeSentiment)
        .tool(SearchTweets)
        .build()
}

#[tool(
    description = "Delegate a task to research agent. It can search the web, analyze pages, view images, and research X (Twitter)"
)]
pub async fn delegate_to_research_agent(prompt: String) -> Result<String> {
    let ctx = SignerContext::current().await;
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_research_agent(ctx.locale()))),
        "research_agent".to_string(),
        ctx,
        false,
    )
    .await
}

// FIXME this could be streamed too
#[tool(
    description = "View an image, accepts an image URL, could be ipfs.io link or a direct URL with .{png,jpg,jpeg,gif,webp}"
)]
pub async fn view_image(image_url: String) -> Result<String> {
    let locale = SignerContext::current().await.locale();
    let agent = gemini_agent_builder()
        .preamble(if locale == "zh" {
            "你是一个乐于助人的助手，可以阅读图像并描述它们"
        } else {
            "You are a helpful assistant that can read images and describe them"
        })
        .build();
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
