use std::{error::Error, str::FromStr, time::Duration};

use crate::{
    constants,
    jito::{self, SearcherClient},
    listener::Listener,
    provider::Provider,
    raydium::{self, calc_result_to_financials, get_burn_pct},
    tx_parser::NewPool,
};
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use log::{info, warn};
use raydium_library::amm;
use serde_json::json;
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking, rpc_client::RpcClient, rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair,
};
use spl_token::state::Mint;

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
    amm_pool: &Pubkey,
) -> Result<bool, Box<dyn Error>> {
    // load amm keys
    let rpc_client = RpcClient::new(dotenv!("RPC_URL").to_string());
    let amm_program =
        Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)
            .expect("amm program");
    let amm_keys =
        amm::utils::load_amm_keys(&rpc_client, &amm_program, amm_pool)?;
    let lp_mint = amm_keys.amm_lp_mint;
    let coin_mint_is_sol = amm_keys.amm_coin_mint
        == Pubkey::from_str(constants::SOLANA_PROGRAM_ID).expect("sol mint");

    let client =
        nonblocking::pubsub_client::PubsubClient::new(dotenv!("WS_URL"))
            .await
            .expect("pubsub client async");
    let (mut stream, unsub) = client
        .account_subscribe(
            &lp_mint,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            }),
        )
        .await
        .expect("subscribe to logs");

    info!("listening for burn for {}", lp_mint.to_string());
    while let Some(log) = stream.next().await {
        info!("log: {:?}", log);
        if let UiAccountData::LegacyBinary(data) = log.value.data {
            let mint_data =
                Mint::unpack(bs58::decode(data).into_vec()?.as_slice())
                    .expect("unpack mint data");
            info!("mint data: {:?}", mint_data);

            let (result, market_keys, _) =
                raydium::get_calc_result(&rpc_client, amm_pool)?;

            // check if any sol pooled before checking burn_pct for correct res
            // rug-pulled tokens have LP supply of 0
            let sol_pooled =
                raydium::calc_result_to_financials(coin_mint_is_sol, result, 0);
            if sol_pooled < 1. {
                let token_mint = if coin_mint_is_sol {
                    market_keys.pc_mint
                } else {
                    market_keys.coin_mint
                };
                warn!("rug pull: {}, sol pooled: {}", token_mint, sol_pooled);
                return Ok(false);
            }

            let burn_pct = get_burn_pct(mint_data, result).expect("burn_pct");
            if burn_pct > 90. {
                info!("burn pct: {}", burn_pct);
                // check here if market cap is right
                if sol_pooled < 20. {
                    warn!("sol pooled: {} < 30", sol_pooled);
                    return Ok(false);
                }
                return Ok(true);
            }
        }
    }

    unsub().await;

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

    let ok = listen_for_burn(&new_pool_info.amm_pool_id).await?;
    if !ok {
        return Ok(());
    }

    // this should be converted to listening as well
    // some tokens have mint and freeze authority disabled a bit later
    // but mostly before the burn
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
