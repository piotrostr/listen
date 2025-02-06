use crate::dexscreener::{search_ticker, DexScreenerResponse};
use anyhow::Result;
use rig_tool_macro::tool;

#[tool(description = "Search for a token on DexScreener")]
pub async fn search_on_dex_screener(
    phrase: String,
) -> Result<DexScreenerResponse> {
    search_ticker(phrase).await
}
