use crate::{
    agents::delegate::delegate_to_agent,
    common::{gemini_agent_builder, GeminiAgent},
    data::listen_api_tools::FetchPriceActionAnalysis,
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

#[tool(
    description = "Delegate a task to chart analysis agent. It can fetch and analyze charts across different timeframes"
)]
pub async fn delegate_to_chart_agent(prompt: String) -> Result<String> {
    let signer = SignerContext::current().await;
    delegate_to_agent(
        prompt,
        create_chart_agent(),
        "chart_agent".to_string(),
        signer,
        false,
    )
    .await
}
