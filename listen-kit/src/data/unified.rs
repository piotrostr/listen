use anyhow::{anyhow, Result};
use privy::caip2::Caip2;
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

use crate::{
    data::{fetch_candlesticks, fetch_token_metadata, Candlestick},
    evm::tools::{get_erc20_balance, get_eth_balance},
    evm_fallback::{validate_chain_id, EvmFallback, SOLANA_CHAIN_ID},
    signer::SignerContext,
    solana::{
        tools::{get_sol_balance, get_spl_token_balance},
        util::validate_mint,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplePriceTick {
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceInfo {
    pub latest_price: f64,
    pub ema_price_ticks: Vec<SimplePriceTick>,
    pub price_ticks_timeframe: String,
    pub total_volume: f64,
    pub pct_change: f64,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub metadata: Option<serde_json::Value>,
    pub price_info: Option<PriceInfo>,
}

#[tool(description = "
Get the core token information - the metadata, socials, recent price action and volume.

Parameters:
- address (string): address of the token to fetch metadata for
- chain_id (string): numeric string of the chain ID of the token to fetch metadata for. Leave blank for Solana tokens.
")]
pub async fn get_token(
    address: String,
    chain_id: Option<String>,
) -> Result<Token> {
    get_token_evm(address, chain_id.unwrap_or(SOLANA_CHAIN_ID.to_string()))
        .await
}

async fn get_token_evm(address: String, chain_id: String) -> Result<Token> {
    let evm_fallback = EvmFallback::from_env()?;
    let pool_address = evm_fallback
        .find_pair_address(&address, chain_id.clone())
        .await?;
    if pool_address.is_none() {
        return Err(anyhow!("No pool address found for token"));
    }

    let pool_address = pool_address.unwrap();

    let (metadata_result, candlesticks_result) = tokio::join!(
        evm_fallback.fetch_token_info(&address, chain_id.clone()),
        evm_fallback.fetch_candlesticks(
            &pool_address,
            chain_id.clone(),
            "15m",
            Some(200)
        )
    );

    let metadata = metadata_result.ok();
    let price_info = match candlesticks_result {
        Ok(candlesticks) => candlesticks_and_timeframe_to_price_info(
            candlesticks,
            "15m".to_string(),
        )
        .ok(),
        Err(_) => None,
    };

    Ok(Token {
        metadata: serde_json::to_value(metadata).ok(),
        price_info,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub balance: String,
    pub address: String,
    pub chain_id: String,
    pub decimals: u8,
}

#[tool(description = "
Get the balance of any token. EVM (ERC20), SVM (SPL) or native (Solana, Ethereum, BNB, etc).

If the token is native to the chain, it will return the balance of the native token.
If the token is an SPL token, it will return the balance of the SPL token.
If the token is an EVM token, it will return the balance of the EVM token.

Parameters:
- address (string): address of the token to fetch balance for, for native tokens, use \"native\" (Solana, Ethereum or BNB)
- chain_id (string): numeric string of the chain ID of the token to fetch balance for.
")]
pub async fn get_token_balance(
    address: String,
    chain_id: String,
) -> Result<TokenBalance> {
    validate_chain_id(chain_id.clone())?;

    if address == "native"
        || address == "solana"
        || address == "So11111111111111111111111111111111111111112"
        || address == "0x0000000000000000000000000000000000000000"
    {
        if chain_id == *SOLANA_CHAIN_ID || chain_id == *Caip2::SOLANA {
            let balance = get_sol_balance().await?;
            return Ok(TokenBalance {
                balance: balance.to_string(),
                address,
                chain_id,
                decimals: 9,
            });
        }

        let evm_wallet_address = SignerContext::current()
            .await
            .address()
            .ok_or(anyhow!("EVM wallet not found"))?;

        let (balance, decimals) =
            get_eth_balance(evm_wallet_address, chain_id.clone()).await?;

        return Ok(TokenBalance {
            balance,
            address,
            chain_id,
            decimals: decimals as u8,
        });
    }

    if chain_id == SOLANA_CHAIN_ID.to_string() {
        let (balance, decimals, address) =
            get_spl_token_balance(address).await?;

        return Ok(TokenBalance {
            balance: balance.to_string(),
            address,
            chain_id,
            decimals,
        });
    }

    let evm_wallet_address = SignerContext::current()
        .await
        .address()
        .ok_or(anyhow!("EVM wallet not found"))?;

    get_erc20_balance(address, evm_wallet_address, chain_id).await
}

#[allow(unused)]
async fn get_token_solana(address: String) -> Result<Token> {
    validate_mint(&address)?;

    let (metadata_result, candlesticks_result) = if address
        == "So11111111111111111111111111111111111111112".to_string()
        || address == "solana".to_string()
    {
        let evm_fallback = EvmFallback::from_env()?;
        tokio::join!(
            fetch_token_metadata(address.clone()),
            evm_fallback.fetch_candlesticks(
                &"Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE",
                SOLANA_CHAIN_ID.to_string(),
                "15m",
                Some(200),
            ),
        )
    } else {
        tokio::join!(
            fetch_token_metadata(address.clone()),
            fetch_candlesticks(address, "15m".to_string())
        )
    };

    let metadata = metadata_result.ok();
    let price_info = match candlesticks_result {
        Ok(candlesticks) => candlesticks_and_timeframe_to_price_info(
            candlesticks,
            "15m".to_string(),
        )
        .ok(),
        Err(_) => None,
    };

    Ok(Token {
        metadata,
        price_info,
    })
}

pub fn candlesticks_and_timeframe_to_price_info(
    mut candlesticks: Vec<Candlestick>,
    timeframe: String,
) -> Result<PriceInfo> {
    if candlesticks.is_empty() {
        return Err(anyhow!("No candlesticks data available"));
    }

    candlesticks.sort_by_key(|c| c.timestamp);

    let period = 5.0;
    let multiplier = 2.0 / (period + 1.0);

    let first = candlesticks.first().expect("Already checked for empty");
    let last = candlesticks.last().expect("Already checked for empty");

    let mut ema_price_ticks = Vec::new();
    let mut current_ema = first.close;
    let total_volume: f64 = candlesticks.iter().map(|c| c.volume).sum();

    // Take fewer points for smoother visualization
    let sample_rate = 4; // adjust this to control smoothness
    for (i, stick) in candlesticks.iter().enumerate() {
        current_ema =
            stick.close * multiplier + current_ema * (1.0 - multiplier);

        if i % sample_rate == 0 {
            ema_price_ticks.push(SimplePriceTick { price: current_ema });
        }
    }

    // Ensure we always include the last point
    if !ema_price_ticks.is_empty()
        && ema_price_ticks.last().unwrap().price != current_ema
    {
        ema_price_ticks.push(SimplePriceTick { price: current_ema });
    }

    let pct_change = ((last.close - first.close) / first.close) * 100.0;
    let duration_secs = last.timestamp - first.timestamp;
    let duration_hours = duration_secs as f64 / 3600.0;
    let period = format!("last {:.1} hours", duration_hours);

    Ok(PriceInfo {
        latest_price: current_ema,
        ema_price_ticks,
        price_ticks_timeframe: timeframe,
        total_volume,
        pct_change,
        period,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ethers::signers::LocalWallet;

    use crate::signer::solana::LocalSolanaSigner;

    use super::*;

    fn make_test_signer_evm() -> LocalWallet {
        let private_key = std::env::var("ETHEREUM_PRIVATE_KEY").unwrap();
        private_key.parse().unwrap()
    }

    fn make_test_signer_sol() -> LocalSolanaSigner {
        let private_key = std::env::var("SOLANA_PRIVATE_KEY").unwrap();
        LocalSolanaSigner::new(private_key)
    }

    #[tokio::test]
    async fn test_get_token_balance_sol_native() {
        let signer = make_test_signer_sol();
        SignerContext::with_signer(Arc::new(signer), async {
            let balance = get_token_balance(
                "native".to_string(),
                SOLANA_CHAIN_ID.to_string(),
            )
            .await;
            println!("{:?}", balance);
            assert!(balance.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_token_balance_evm_native() {
        let signer = make_test_signer_evm();
        SignerContext::with_signer(Arc::new(signer), async {
            let balance =
                get_token_balance("native".to_string(), "1".to_string())
                    .await;
            println!("{:?}", balance);
            assert!(balance.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_token_balance_evm_erc20() {
        let signer = make_test_signer_evm();
        SignerContext::with_signer(Arc::new(signer), async {
            let pepe = "0x6982508145454Ce325dDbE47a25d4ec3d2311933";
            let balance =
                get_token_balance(pepe.to_string(), "1".to_string()).await;
            println!("{:?}", balance);
            assert!(balance.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_token_balance_sol_spl() {
        let signer = make_test_signer_sol();
        SignerContext::with_signer(Arc::new(signer), async {
            let fartcoin = "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump";
            let balance = get_token_balance(
                fartcoin.to_string(),
                SOLANA_CHAIN_ID.to_string(),
            )
            .await;
            println!("{:?}", balance);
            assert!(balance.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_token_sol() {
        let address = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
        let token =
            get_token(address.to_string(), Some(SOLANA_CHAIN_ID.to_string()))
                .await;
        println!("{:?}", token);
        assert!(token.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_balance_sol_caip2() {
        let chain_id = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
        let address = "So11111111111111111111111111111111111111112";
        let signer = make_test_signer_sol();
        SignerContext::with_signer(Arc::new(signer), async {
            let balance =
                get_token_balance(address.to_string(), chain_id.to_string())
                    .await;
            println!("{:?}", balance);
            assert!(balance.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    }
}
