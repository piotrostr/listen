use crate::common::{gemini_agent_builder, GeminiAgent};
use anyhow::Result;

pub async fn create_solana_trader_agent() -> Result<GeminiAgent> {
    let agent = gemini_agent_builder().build();
    Ok(agent)
}
