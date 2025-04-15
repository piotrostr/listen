use crate::agents::delegate::delegate_to_agent;
use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::twitter_tools::{
    FetchXPost, ResearchXProfile, SearchTweets,
};
use crate::reasoning_loop::Model;
use crate::signer::SignerContext;
use anyhow::Result;
use rig_tool_macro::tool;
use std::sync::Arc;

const PREAMBLE_EN: &str = "You are a deep X research agent. Your goal is to perform thorough recursive analysis:
        1. For each tool call result, analyze if there are more leads to explore
        2. If you find new profiles, posts, or topics, investigate them
        3. Build a comprehensive picture by following all relevant leads
        4. Don't stop at surface-level information - dig deeper into each finding
        5. If you find something interesting, use other tools to verify and expand on it. Always use English.";

const PREAMBLE_ZH: &str =
    "你是一个深入的X研究代理。你的目标是进行彻底的递归分析：
        1. 对于每个工具调用结果，分析是否存在更多需要探索的线索
        2. 如果你发现新的个人资料、帖子或话题，调查它们
        3. 通过遵循所有相关线索建立全面图景
        4. 不要停留在表面信息 - 深入挖掘每个发现
        5. 如果你发现有趣的东西，使用其他工具验证并扩展它. 请使用中文";

pub fn create_x_agent(locale: String) -> GeminiAgent {
    gemini_agent_builder()
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .build()
}

#[tool(description = "Delegate a task to x agent")]
pub async fn delegate_to_x_agent(prompt: String) -> Result<String> {
    let signer = SignerContext::current().await;
    let user_id = signer.user_id().unwrap_or_default();
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_x_agent(signer.locale()))),
        "x_agent".to_string(),
        signer,
        false,
        user_id,
    )
    .await
}
