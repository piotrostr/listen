use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::listen_api_tools::FetchTokenMetadata;
use crate::solana::tools::{GetSolBalance, GetSplTokenBalance};

pub fn create_on_chain_analytics_agent() -> GeminiAgent {
    gemini_agent_builder().preamble("You are a deep on-chain research agent. Your goal is to perform thorough recursive analysis:
    1. For each tool call result, analyze if there are more leads to explore
    2. If you find new addresses, tokens, or entities, investigate them
    3. Build a comprehensive picture by following all relevant leads
    4. Don't stop at surface-level information - dig deeper into each finding
    5. If you find something interesting, use other tools to verify and expand on it")
        .tool(FetchTokenMetadata)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .build()
}
