#![allow(non_upper_case_globals)]

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reqwest::Client;
use rig_tool_macro::tool;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data::PortfolioItem;
use crate::dexscreener::PairInfo;
use crate::util::wrap_unsafe;

static KEYPAIR: Lazy<Arc<RwLock<Keypair>>> =
    Lazy::new(|| Arc::new(RwLock::new(Keypair::new())));

static RPC_URL: Lazy<String> = Lazy::new(|| {
    dotenv::dotenv().ok();
    std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
});

pub async fn initialize(private_key: String) {
    let keypair = Keypair::from_base58_string(&private_key);

    *KEYPAIR.write().await = keypair;
}

fn create_rpc() -> RpcClient {
    RpcClient::new(RPC_URL.to_string())
}

#[tool]
pub async fn trade(
    input_mint: String,
    input_amount: f64,
    output_mint: String,
    slippage_bps: u16,
) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::trade::trade(
            input_mint,
            sol_to_lamports(input_amount),
            output_mint,
            slippage_bps,
            &keypair,
        )
        .await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

#[tool]
pub async fn transfer_sol(to: String, amount: u64) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::transfer::transfer_sol(Pubkey::from_str(&to)?, amount, &keypair)
            .await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

/// param amount is token amount, accounting for decimals
/// e.g. 1 Fartcoin = 1 * 10^6 (6 decimals)
#[tool]
pub async fn transfer_token(
    to: String,
    amount: u64,
    mint: String,
) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::transfer::transfer_spl(
            Pubkey::from_str(&to)?,
            amount,
            Pubkey::from_str(&mint)?,
            &keypair,
            &create_rpc(),
        )
        .await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

#[tool]
pub async fn wallet_address() -> Result<String> {
    let keypair = KEYPAIR.read().await;
    Ok(keypair.pubkey().to_string())
}

#[tool]
pub async fn get_balance() -> Result<u64> {
    let pubkey = KEYPAIR.read().await.pubkey();

    wrap_unsafe(move || async move {
        create_rpc()
            .get_balance(&pubkey)
            .await
            .map_err(|e| anyhow!("{:#?}", e))
    })
    .await
}

/// get_token_balance returns the amount as String and the decimals as u8
/// in order to convert to UI amount: amount / 10^decimals
#[tool]
pub async fn get_token_balance(mint: String) -> Result<(String, u8)> {
    let keypair = KEYPAIR.read().await;
    let mint = Pubkey::from_str(&mint)?;
    let ata = spl_associated_token_account::get_associated_token_address(
        &keypair.pubkey(),
        &mint,
    );
    let balance = wrap_unsafe(move || async move {
        create_rpc()
            .get_token_account_balance(&ata)
            .await
            .map_err(|e| anyhow!("{:#?}", e))
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))?;

    Ok((balance.amount, balance.decimals))
}

#[tool]
#[allow(clippy::too_many_arguments)]
pub async fn deploy_token(
    name: String,
    symbol: String,
    twitter: String,
    website: String,
    dev_buy: u64,
    telegram: String,
    image_url: String,
    description: String,
) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::deploy_token::deploy_token(
            crate::deploy_token::DeployTokenParams {
                name,
                symbol,
                twitter: Some(twitter),
                website: Some(website),
                dev_buy: Some(dev_buy),
                telegram: Some(telegram),
                image_url: Some(image_url),
                description,
            },
            &keypair,
        )
        .await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

#[tool]
pub async fn fetch_token_price(mint: String) -> Result<f64> {
    crate::price::fetch_token_price(mint, &Client::new()).await
}

#[tool]
pub async fn buy_pump_token(
    mint: String,
    sol_amount: f64,
    slippage_bps: u16,
) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::trade_pump::buy_pump_fun(
            mint,
            sol_to_lamports(sol_amount),
            slippage_bps,
            &create_rpc(),
            &keypair,
        )
        .await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

#[tool]
pub async fn sell_pump_token(
    mint: String,
    token_amount: u64,
) -> Result<String> {
    let keypair = KEYPAIR.read().await;
    wrap_unsafe(move || async move {
        crate::trade_pump::sell_pump_fun(mint, token_amount, &keypair).await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}

#[tool]
pub async fn portfolio() -> Result<Vec<PortfolioItem>> {
    let keypair = KEYPAIR.read().await;
    let holdings = wrap_unsafe(move || async move {
        crate::balance::get_holdings(&create_rpc(), &keypair.pubkey())
            .await
            .map_err(|e| anyhow!("{:#?}", e))
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))?;

    crate::data::holdings_to_portfolio(holdings).await
}

#[tool]
pub async fn fetch_pair_info(mint_or_symbol: String) -> Result<PairInfo> {
    crate::data::fetch_pair_info(mint_or_symbol).await
}

#[tool]
pub async fn scan(mint: String) -> Result<String> {
    crate::scan::scan(mint).await
}
