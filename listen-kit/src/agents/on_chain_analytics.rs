use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::listen_api_tools::FetchTokenMetadata;
use crate::solana::tools::{GetSolBalance, GetSplTokenBalance};
use anyhow::Result;

pub fn create_on_chain_analytics_agent() -> GeminiAgent {
    gemini_agent_builder().preamble("Run onchain analytics, use your tools to get as much output for the given prompt as possible. If a tool output yields more outputs, continue to explore")
        .tool(FetchTokenMetadata)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .build()
}
