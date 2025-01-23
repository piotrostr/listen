use anyhow::Result;

use crate::dexscreener::{search_ticker, PairInfo};

pub async fn fetch_pair_info(mint_or_symbol: String) -> Result<PairInfo> {
    let res = search_ticker(mint_or_symbol.clone()).await?;

    let mut matching_pairs: Vec<&PairInfo> = res
        .pairs
        .iter()
        .filter(|pair| pair.chain_id == "solana")
        .collect();

    matching_pairs.sort_by(|a, b| {
        b.volume
            .h24
            .partial_cmp(&a.volume.h24)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // get the pair with the highest liquidity
    matching_pairs
        .first()
        .map(|pair| (*pair).clone()) // Dereference and clone the PairInfo
        .ok_or_else(|| {
            anyhow::anyhow!("No matching pairs found for {}", mint_or_symbol)
        })
}
