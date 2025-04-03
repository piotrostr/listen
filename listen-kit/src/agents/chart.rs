use std::sync::Arc;

use crate::{
    agents::delegate::delegate_to_agent,
    common::{gemini_agent_builder, GeminiAgent},
    data::listen_api_tools::FetchPriceActionAnalysis,
    reasoning_loop::Model,
    signer::SignerContext,
};
use anyhow::Result;
use rig_tool_macro::tool;

const PREAMBLE_EN: &str = "You are a deep chart analysis agent. Your goal is to perform thorough technical analysis:
        1. For each price action analysis, look for significant patterns and signals
        2. If you find interesting price movements, investigate the timeframes around them
        3. Build a comprehensive picture by analyzing multiple technical indicators
        4. Don't stop at surface-level patterns - dig deeper into each finding
        5. If you find something interesting, verify it against other timeframes and indicators";

const PREAMBLE_ZH: &str =
    "你是一个深入的图表分析代理。你的目标是进行彻底的技术分析：
        1. 对于每个价格行动分析，寻找重要的模式和信号
        2. 如果发现有趣的价格变动，调查它们周围的时间框架
        3. 通过分析多个技术指标建立全面图景
        4. 不要停留在表面模式 - 深入挖掘每个发现
        5. 如果你发现有趣的东西，验证它与其他时间框架和其他指标的一致性";

pub fn create_chart_agent(locale: String) -> GeminiAgent {
    gemini_agent_builder()
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .tool(FetchPriceActionAnalysis)
        .build()
}

#[tool(
    description = "Delegate a task to chart analysis agent. It can fetch and analyze charts across different timeframes"
)]
pub async fn delegate_to_chart_agent(prompt: String) -> Result<String> {
    let ctx = SignerContext::current().await;
    delegate_to_agent(
        prompt,
        Model::Gemini(Arc::new(create_chart_agent(ctx.locale()))),
        "chart_agent".to_string(),
        ctx,
        false,
    )
    .await
}
