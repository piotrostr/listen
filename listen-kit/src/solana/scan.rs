use std::str::FromStr;

use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

use crate::solana::pump::fetch_metadata;

pub async fn scan(mint: String) -> Result<String> {
    let pubkey = match Pubkey::from_str(&mint) {
        Ok(pubkey) => pubkey,
        Err(e) => return Err(anyhow!(e.to_string())),
    };
    let metadata = fetch_metadata(&pubkey).await?;
    // could check the deploy history of creator here too
    // let _raw_response = vec![];
    if let Some(twitter) = metadata.twitter {
        let res = reqwest::get(twitter).await?.text().await?;
        tracing::debug!(?res, "scan:twitter");
    }
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey;

    use super::*;

    #[tokio::test]
    #[ignore]
    pub async fn test_scan() {
        let pubkey = pubkey!("KENJSUYLASHUMfHyy5o4Hp2FdNqZg1AsUPhfH2kYvEP");
        scan(pubkey.to_string()).await.unwrap();
    }
}
