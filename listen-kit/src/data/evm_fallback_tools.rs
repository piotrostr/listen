use crate::{
    common::spawn_with_signer_and_channel,
    data::TopToken,
    distiller::analyst::Analyst,
    evm_fallback::{token_info::GtTokenMetadata, EvmFallback},
    reasoning_loop::ReasoningLoop,
    signer::SignerContext,
};
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;

#[tool(description = "
Fetch token metadata for any EVM token from the GeckoTerminal API.

Parameters:
- address (string): The address of the token to fetch metadata for
- chain_id (u64): The chain ID of the token

")]
pub async fn fetch_token_metadata_evm(
    address: String,
    chain_id: u64,
) -> Result<GtTokenMetadata> {
    let evm_fallback = EvmFallback::from_env()?;
    let token_info =
        evm_fallback.fetch_token_info(&address, chain_id).await?;
    Ok(token_info)
}

#[tool(description = "
Fetch token price for any EVM token from the GeckoTerminal API.

Parameters:
- pair_address (string): The address of the token LP pair to fetch price for -
this is different from the token address, the LP address can be found through
searching on DexScreener.
- chain_id (u64): The chain ID of the token
- interval (string): The candlestick interval, one of:
  * '15m' (15 minutes)
  * '30m' (30 minutes)
  * '1h'  (1 hour)
  * '4h'  (4 hours)
  * '1d'  (1 day)
- intent (string): The intent of the analysis, passed on to the Chart Analyst agent, possible to pass \"\" for no intent
")]
pub async fn fetch_price_action_analysis_evm(
    pair_address: String,
    chain_id: u64,
    interval: String,
    intent: String,
) -> Result<String> {
    let evm_fallback = EvmFallback::from_env()?;
    let candlesticks = evm_fallback
        .fetch_candlesticks(&pair_address, chain_id, &interval, Some(200))
        .await?;
    let ctx = SignerContext::current().await;
    let locale = ctx.locale();
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|e| anyhow!("Failed to create Analyst: {}", e))?;

    let channel = ReasoningLoop::get_current_stream_channel().await;

    spawn_with_signer_and_channel(ctx, channel, move || async move {
        analyst
            .analyze_chart(&candlesticks, &interval, Some(intent))
            .await
            .map_err(|e| anyhow!("Failed to analyze chart: {}", e))
    })
    .await
    .await?
}
#[tool(description = "
Fetch top tokens by chain ID from the GeckoTerminal API. 

Use this tool to find trending tokens on EVM chains: Eth Mainnet, Base, Arbitrum or Binance Smart Chain.

Parameters:
- chain_id (u64): The chain ID of the tokens to fetch
- limit (string): number of tokens to return; \"6\" is a good limit, unless specified otherwise
- duration (string): duration of the timeframe to fetch, one of:
  * 5m (5 minutes)
  * 1h (1 hour)
  * 6h (6 hours)
  * 24h (24 hours)
  if not specified, \"6h\" is good

Returns a list of top tokens with their market data.
")]
pub async fn fetch_top_tokens_by_chain_id(
    chain_id: u64,
    limit: String,
    duration: String,
) -> Result<Vec<TopToken>> {
    let evm_fallback = EvmFallback::from_env()?;
    let tokens = evm_fallback
        .fetch_top_tokens(chain_id, duration, limit.parse::<usize>()?)
        .await?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::solana::util::make_test_signer;

    use super::*;

    #[tokio::test]
    async fn test_fetch_price_action_analysis() {
        SignerContext::with_signer(make_test_signer(), async {
            let analysis = fetch_price_action_analysis_evm(
                "0x4e829F8A5213c42535AB84AA40BD4aDCCE9cBa02".to_string(),
                8453, // base
                "5m".to_string(),
                "".to_string(),
            )
            .await
            .unwrap();
            tracing::info!("{:?}", analysis);
            Ok(())
        })
        .await
        .unwrap();
    }
}
