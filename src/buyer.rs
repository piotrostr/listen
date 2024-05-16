use std::error::Error;

use crate::{
    constants,
    jito::{self, SearcherClient},
    listener::Listener,
    provider::Provider,
    raydium,
    tx_parser::NewPool,
};
use dotenv_codegen::dotenv;
use log::{info, warn};
use serde_json::json;
use solana_client::nonblocking;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

// Trader is a wrapper to listen on liquidity burn of a new listing
// plus verify that the supply is not centralized, and perform same sanity
// checks as listener
// this is to separate out listening and parsing too and enable off-line
// processing
// ideally it would also track position and sell at the right time
pub struct Trader {}

// TODO check for top holders

// listen_for_burn listens until the liquidity is burnt or a rugpull happens
pub async fn listen_for_burn(
    listener: &Listener,
    provider: &Provider,
    mint: &Pubkey,
    amm_pool: &Pubkey,
) -> Result<bool, Box<dyn Error>> {
    let (subs, receiver) = listener.account_subscribe(mint).unwrap();
    while let Ok(_) = receiver.recv() {
        let result = raydium::get_calc_result(&provider.rpc_client, amm_pool)
            .expect("get calc result");
        let burn_pct =
            raydium::get_burn_pct(&provider.rpc_client, mint, result)
                .expect("get burn pct");
        if burn_pct > 90. {
            info!("Burn pct is over 90%, ready to trade");
            return Ok(true);
        }
        if result.pool_lp_amount as f64 / 9f64 < 1. {
            warn!("rugpull of {}", mint.to_string());
            return Ok(false);
        }
    }

    subs.send_unsubscribe()?;

    Ok(false)
}

pub async fn handle_new_pair(
    new_pool_info: NewPool,
    amount: u64,
    slippage: u64,
    wallet: &Keypair,
    provider: &Provider,
    searcher_client: &mut SearcherClient,
) -> Result<(), Box<dyn Error>> {
    let mint = if new_pool_info.input_mint.to_string()
        == constants::SOLANA_PROGRAM_ID
    {
        new_pool_info.output_mint
    } else {
        new_pool_info.input_mint
    };

    let (is_safe, msg) =
        provider.sanity_check(&mint).await.expect("sanity check");
    if !is_safe {
        warn!("Unsafe pool: {}, skipping", msg);
        return Ok(());
    }

    let swap_context = raydium::make_swap_context(
        provider,
        new_pool_info.amm_pool_id,
        new_pool_info.input_mint,
        new_pool_info.output_mint,
        wallet,
        slippage,
        amount,
    )
    .await
    .expect("makes swap context");

    let result =
        raydium::get_calc_result(&provider.rpc_client, &swap_context.amm_pool)?;
    let coin_mint_is_sol = swap_context.market_keys.coin_mint.to_string()
        == constants::SOLANA_PROGRAM_ID;
    let pooled_sol =
        raydium::calc_result_to_financials(coin_mint_is_sol, result, 0);
    if pooled_sol < 80. {
        info!("pooled sol: {} is below the 100. thresh", pooled_sol);
        return Ok(());
    }

    let listener = Listener::new(dotenv!("WS_URL").to_string());
    let ok = listen_for_burn(
        &listener,
        provider,
        &new_pool_info.input_mint,
        &swap_context.amm_pool,
    )
    .await?;
    if !ok {
        return Ok(());
    }

    let start = std::time::Instant::now();
    let quick = true;
    let mut ixs =
        raydium::make_swap_ixs(provider, wallet, &swap_context, quick)
            .expect("make swap ixs");

    info!("took {:?} to pack", start.elapsed());

    let tip = 50000;

    let rpc_client = &nonblocking::rpc_client::RpcClient::new(
        dotenv!("RPC_URL").to_string(),
    );
    let swap_result =
        jito::send_swap_tx(&mut ixs, tip, wallet, searcher_client, &rpc_client)
            .await;

    info!(
        "{}",
        serde_json::to_string_pretty(&json!(vec![
            format!("https://dexscreener.com/solana/{}", mint.to_string()),
            format!("https://jup.ag/swap/{}-SOL", mint.to_string()),
            format!("https://rugcheck.xyz/tokens/{}", mint.to_string()),
        ]))
        .expect("to string pretty")
    );

    info!("swap result: {:?}", swap_result);

    Ok(())
}
