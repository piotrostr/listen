use anyhow::{anyhow, Result};
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TopToken;

#[derive(Debug, Serialize, Deserialize)]
struct JupiterTokenData {
    id: String,
    name: String,
    symbol: String,
    icon: String,
    decimals: u8,
    dev: String,
    #[serde(rename = "circSupply")]
    circ_supply: f64,
    #[serde(rename = "totalSupply")]
    total_supply: f64,
    #[serde(rename = "tokenProgram")]
    token_program: String,
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    #[serde(rename = "freezeAuthority")]
    freeze_authority: String,
    #[serde(rename = "firstPool")]
    first_pool: Value,
    #[serde(rename = "holderCount")]
    holder_count: u64,
    audit: Value,
    #[serde(rename = "stockData")]
    stock_data: StockData,
    #[serde(rename = "organicScore")]
    organic_score: f64,
    #[serde(rename = "organicScoreLabel")]
    organic_score_label: String,
    #[serde(rename = "isVerified")]
    is_verified: bool,
    cexes: Vec<String>,
    tags: Vec<String>,
    fdv: f64,
    mcap: f64,
    #[serde(rename = "usdPrice")]
    usd_price: f64,
    #[serde(rename = "priceBlockId")]
    price_block_id: u64,
    liquidity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stats5m: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stats1h: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stats6h: Option<Value>,
    stats24h: Stats24h,
    #[serde(rename = "ctLikes")]
    ct_likes: u64,
    #[serde(rename = "smartCtLikes")]
    smart_ct_likes: u64,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StockData {
    price: f64,
    mcap: f64,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stats24h {
    #[serde(rename = "priceChange")]
    price_change: f64,
    #[serde(rename = "holderChange")]
    holder_change: f64,
    #[serde(rename = "liquidityChange")]
    liquidity_change: f64,
    #[serde(rename = "volumeChange")]
    volume_change: f64,
    #[serde(rename = "buyVolume")]
    buy_volume: f64,
    #[serde(rename = "sellVolume")]
    sell_volume: f64,
    #[serde(rename = "buyOrganicVolume")]
    buy_organic_volume: f64,
    #[serde(rename = "sellOrganicVolume")]
    sell_organic_volume: f64,
    #[serde(rename = "numBuys")]
    num_buys: u64,
    #[serde(rename = "numSells")]
    num_sells: u64,
    #[serde(rename = "numTraders")]
    num_traders: u64,
    #[serde(rename = "numOrganicBuyers")]
    num_organic_buyers: u64,
    #[serde(rename = "numNetBuyers")]
    num_net_buyers: u64,
}

const STOCK_ADDRESSES: &[(&str, &str)] = &[
    ("Circle", "XsueG8BtpquVJX9LVLLEGuViXUungE6WmK5YZ3p3bd1"),
    ("NVIDIA", "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh"),
    ("Tesla", "XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB"),
    (
        "MicroStrategy",
        "XsP7xzNPvEHS1m6qfanPUGjNmdnmsLKEoNAnHjdxxyZ",
    ),
    ("Alphabet", "XsCPL9dNWBMvFtTmwcCA5v3xWPSMEBCszbQdiLLq6aN"),
    ("Apple", "XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp"),
    ("Robinhood", "XsvNBAYkrDRNhA7wPHQfX3ZUXZyZLdnCQDfHZ56bzpg"),
    ("Amazon", "Xs3eBt7uRfJX8QUs4suhyU8p2M6DoUDrJyWBa8LLZsg"),
    ("Coinbase", "Xs7ZdzSHLU9ftNJsii5fCeJhoRWSC32SQGzGQtePxNu"),
    (
        "Meta Platforms",
        "Xsa62P5mvPszXL1krVUnU5ar38bBSVcWAB6fmPCo5Zu",
    ),
    ("S&P 500 ETF", "XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W"),
    ("Nasdaq-100", "Xs8S1uUs1zvS2p7iwtsG3b6fkhpvmwz4GYU3gWAmWHZ"),
];

#[tool(description = "
Fetch top stock tokens from Jupiter API. This fetches tradable real-world stock representations on Solana.

Returns a list of top stocks including:
- CRCL (Circle)
- NVDA (NVIDIA)
- TSLA (Tesla)
- MSTR (MicroStrategy)
- GOOGL (Alphabet)
- AAPL (Apple)
- HOOD (Robinhood)
- AMZN (Amazon)
- COIN (Coinbase)
- META (Meta Platforms)
- SPY (S&P 500 ETF)
- QQQ (Nasdaq-100)

Each stock includes price, market cap, 24h volume, and price change data.
")]
pub async fn fetch_top_stocks() -> Result<Vec<TopToken>> {
    let mut stocks = Vec::new();

    for (name, address) in STOCK_ADDRESSES {
        let url = format!(
            "https://lite-api.jup.ag/tokens/v2/search?query={}",
            address
        );

        let response = match reqwest::get(&url).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to fetch stock data for {}: {}", name, e);
                continue;
            }
        };

        let data: Vec<JupiterTokenData> = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse JSON for {}: {}", name, e);
                continue;
            }
        };

        if let Some(stock_data) = data.first() {
            let total_volume = stock_data.stats24h.buy_volume
                + stock_data.stats24h.sell_volume;

            stocks.push(TopToken {
                name: stock_data.name.clone(),
                pubkey: stock_data.id.clone(),
                price: stock_data.usd_price,
                market_cap: stock_data.mcap,
                volume_24h: total_volume,
                price_change_24h: stock_data.stats24h.price_change,
                chain_id: None,
                pools: Vec::new(),
                img_url: Some(stock_data.icon.clone()),
            });
        }
    }

    if stocks.is_empty() {
        return Err(anyhow!("Failed to fetch any stock data"));
    }

    Ok(stocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_top_stocks() {
        let result = fetch_top_stocks().await;
        if let Err(e) = &result {
            eprintln!("Error fetching stocks: {:?}", e);
        }
        assert!(result.is_ok());

        let stocks = result.unwrap();
        assert!(!stocks.is_empty()); // We expect at least some stocks

        // Verify each stock has valid data
        for stock in &stocks {
            assert!(!stock.name.is_empty());
            assert!(!stock.pubkey.is_empty());
            assert!(stock.price > 0.0);
            assert!(stock.market_cap > 0.0);
        }

        let pretty_string = serde_json::to_string_pretty(&stocks).unwrap();
        println!("Stocks: {}", pretty_string);
    }

    #[tokio::test]
    async fn test_coinbase_stock_specifically() {
        let url = "https://lite-api.jup.ag/tokens/v2/search?query=Xs7ZdzSHLU9ftNJsii5fCeJhoRWSC32SQGzGQtePxNu";

        let response = reqwest::get(url).await.unwrap();
        let text = response.text().await.unwrap();
        println!("Raw response: {}", &text[..500.min(text.len())]);

        let result: Result<Vec<JupiterTokenData>, _> =
            serde_json::from_str(&text);
        match result {
            Ok(data) => println!("Successfully parsed {} items", data.len()),
            Err(e) => println!("Failed to parse: {}", e),
        }
    }
}
