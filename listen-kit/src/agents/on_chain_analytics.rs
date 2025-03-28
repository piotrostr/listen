use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::listen_api_tools::FetchTokenMetadata;
use crate::solana::tools::{GetSolBalance, GetSplTokenBalance};
use anyhow::Result;

pub async fn create_on_chain_analytics_agent() -> Result<GeminiAgent> {
    let agent = gemini_agent_builder()
        .tool(FetchTokenMetadata)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .build();
    Ok(agent)
}
