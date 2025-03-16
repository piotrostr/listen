pub mod tools;
pub mod types;

use anyhow::Result;
use reqwest::Client;
use types::DexScreenerResponse;

pub async fn search_ticker(ticker: String) -> Result<DexScreenerResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.dexscreener.com/latest/dex/search/?q={}&limit=8",
        ticker
    );

    let response = client.get(&url).send().await?;

    if response.status().is_client_error() {
        let res = response.text().await?;
        tracing::error!("Error: {:?}", res);
        return Err(anyhow::anyhow!("Error: {:?}", res));
    }

    let data: serde_json::Value = response.json().await?;

    let mut dex_response: DexScreenerResponse = serde_json::from_value(data)?;

    // Sort by a combined score of liquidity and volume
    dex_response.pairs.sort_by(|a, b| {
        // Get liquidity values (default to 0.0 if not available)
        let a_liq = if a.liquidity.is_some() {
            a.liquidity.as_ref().unwrap().usd.unwrap_or(0.0)
        } else {
            0.0
        };

        let b_liq = if b.liquidity.is_some() {
            b.liquidity.as_ref().unwrap().usd.unwrap_or(0.0)
        } else {
            0.0
        };

        // Get 24h volume values (default to 0.0 if not available)
        let a_vol = if a.volume.is_some() {
            a.volume.as_ref().unwrap().h24.unwrap_or(0.0)
        } else {
            0.0
        };

        let b_vol = if b.volume.is_some() {
            b.volume.as_ref().unwrap().h24.unwrap_or(0.0)
        } else {
            0.0
        };

        // Normalize the values to prevent one metric from dominating
        // We'll use a simple combined score: liquidity + volume/100
        // This gives liquidity more weight but still considers volume
        let a_score = a_liq + (a_vol / 100.0);
        let b_score = b_liq + (b_vol / 100.0);

        // Compare in reverse order for descending sort
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // take top 5
    dex_response.pairs.truncate(5);

    Ok(dex_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_ticker_base() {
        let response = search_ticker("brett".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_search_ticker() {
        let response = search_ticker("BONK".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        println!("{:#?}", response);
    }

    #[tokio::test]
    async fn test_search_by_mint() {
        tracing::debug!("search_by_mint");
        let response = search_ticker(
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump".to_string(),
        )
        .await
        .unwrap();
        tracing::debug!(?response, "search_by_mint");
        assert_eq!(response.schema_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_search_ticker_by_phrase() {
        let response = search_ticker("TheLion".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        println!("{:?}", response);
    }
}
