use crate::dexscreener::{search_ticker, DexScreenerResponse};
use anyhow::Result;
use rig_tool_macro::tool;

#[tool(description = "
Search for a token on DexScreener
phrase: the phrase to search for, could be a token symbol, a ticker, a name, etc.

use it if you dont know the contract address of a token

returns: a list of pairs that match the search phrase, the liquidity, volume, price, and other metrics
")]
pub async fn search_on_dex_screener(
    phrase: String,
) -> Result<DexScreenerResponse> {
    if phrase.starts_with("$") {
        search_ticker(phrase.replace("$", "").to_lowercase()).await
    } else {
        search_ticker(phrase).await
    }
}
