use std::{error::Error, str::FromStr};

use crate::{
    constants,
    jito::{self, SearcherClient},
    provider::Provider,
    raydium::{self, get_burn_pct},
    tx_parser::NewPool,
};
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use jito_searcher_client::token_authenticator;
use log::{debug, info, warn};
use raydium_library::amm;
use serde::Serialize;
use serde_json::json;
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair,
};
use spl_token::state::Mint;
use tokio::fs::OpenOptions;

#[derive(Debug, Serialize, Default)]
pub struct TokenResult {
    pub creation_signature: String,
    pub slot_received: u64,
    pub timestamp_received: String,
    pub timestamp_finalized: String,
    pub mint: String,
    pub amm_pool: String,
    pub outcome: String,
    pub sol_pooled: Option<f64>,
    pub burn_pct: Option<f64>,
    pub top_10_holders: Option<f64>,
}

pub async fn check_top_holders(
    mint: &Pubkey,
    provider: &Provider,
) -> Result<(f64, bool), Box<dyn Error>> {
    let top_holders =
        provider.rpc_client.get_token_largest_accounts(mint).await?;
    let up_to_ten = 10.min(top_holders.len());
    let top_holders = top_holders[0..up_to_ten].to_vec();
    let top_holders_len = top_holders.len();
    let total_supply = provider
        .rpc_client
        .get_token_supply(mint)
        .await?
        .ui_amount
        .unwrap();
    let mut total = 0f64;
    let mut got_raydium = false;
    let mut raydium_holding = 0f64;
    for holder in top_holders {
        debug!("holder: {:?}, balance: {:?}", holder.address, holder.amount);
        if !got_raydium {
            let account_info = provider
                .rpc_client
                .get_token_account_with_commitment(
                    &Pubkey::from_str(holder.address.as_str())?,
                    CommitmentConfig::confirmed(),
                )
                .await?;
            if account_info.value.unwrap().owner
                == constants::RAYDIUM_AUTHORITY_V4_PUBKEY
            {
                raydium_holding = holder.amount.ui_amount.unwrap();
                got_raydium = true;
            }
        }
        total += holder.amount.ui_amount.unwrap();
    }

    total -= raydium_holding;

    debug!(
        "{} top {} holders: {}, raydium: {}",
        mint.to_string(),
        top_holders_len,
        total / total_supply,
        raydium_holding / total_supply
    );

    let top_10_holders = total / total_supply;
    if top_10_holders > 0.25 {
        warn!(
            "{}: centralized supply: {} / {} = {}",
            mint.to_string(),
            total,
            total_supply,
            top_10_holders
        );
        return Ok((top_10_holders, false));
    }

    Ok((top_10_holders, true))
}

pub async fn listen_for_sol_pooled(
    amm_pool: &Pubkey,
    rpc_client: &RpcClient,
    pubsub_client: &PubsubClient,
) -> Result<(f64, bool), Box<dyn Error>> {
    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            amm_pool,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            }),
        )
        .await
        .expect("subscribe to account");

    info!("listening for sol pooled for pool {}", amm_pool.to_string());
    while let Some(_) = stream.next().await {
        let (result, _, amm_keys) =
            raydium::get_calc_result(&rpc_client, amm_pool).await?;
        let coin_mint_is_sol = amm_keys.amm_coin_mint
            == Pubkey::from_str(constants::SOLANA_PROGRAM_ID)
                .expect("sol mint");
        let token_mint = if coin_mint_is_sol {
            amm_keys.amm_pc_mint
        } else {
            amm_keys.amm_coin_mint
        };
        let sol_pooled =
            raydium::calc_result_to_financials(coin_mint_is_sol, result, 0);
        if sol_pooled > 50. {
            info!("{} sol pooled: {}", token_mint, sol_pooled);
            return Ok((sol_pooled, true));
        } else {
            warn!("{} sol pooled: {}", token_mint, sol_pooled);
            return Ok((sol_pooled, false));
        }
    }

    unsub().await;

    Ok((-1., false))
}

// listen_for_burn listens until the liquidity is burnt or a rugpull happens
pub async fn listen_for_burn(
    amm_pool: &Pubkey,
    rpc_client: &RpcClient,
    pubsub_client: &PubsubClient,
) -> Result<(f64, bool), Box<dyn Error>> {
    // load amm keys
    let amm_program =
        Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)
            .expect("amm program");
    let amm_keys =
        amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    let lp_mint = amm_keys.amm_lp_mint;
    let coin_mint_is_sol = amm_keys.amm_coin_mint
        == Pubkey::from_str(constants::SOLANA_PROGRAM_ID).expect("sol mint");

    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            &lp_mint,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            }),
        )
        .await
        .expect("subscribe to account");

    let token_mint = if coin_mint_is_sol {
        amm_keys.amm_pc_mint
    } else {
        amm_keys.amm_coin_mint
    };

    info!("listening for burn for {}", token_mint.to_string());
    while let Some(log) = stream.next().await {
        debug!("log: {:?}", log);
        if let UiAccountData::LegacyBinary(data) = log.value.data {
            let mint_data =
                Mint::unpack(bs58::decode(data).into_vec()?.as_slice())
                    .expect("unpack mint data");
            debug!("mint data: {:?}", mint_data);

            let (result, _, _) =
                raydium::get_calc_result(rpc_client, amm_pool).await?;

            // check if any sol pooled before checking burn_pct for correct res
            // rug-pulled tokens have LP supply of 0
            let sol_pooled =
                raydium::calc_result_to_financials(coin_mint_is_sol, result, 0);
            if sol_pooled < 1. {
                warn!("{} rug pull, sol pooled: {}", token_mint, sol_pooled);
                return Ok((-1., false));
            }

            let burn_pct = get_burn_pct(mint_data, result).expect("burn_pct");
            if burn_pct > 90. {
                info!("burn pct: {}", burn_pct);
                if sol_pooled < 50. {
                    warn!("{} sol pooled: {} < 50", token_mint, sol_pooled);
                    return Ok((-1., false));
                }
                return Ok((burn_pct, true));
            }
        }
    }

    unsub().await;

    Ok((-1., false))
}

pub async fn handle_new_pair(
    new_pool_info: NewPool,
    amount: u64,
    slippage: u64,
    wallet: &Keypair,
    provider: &Provider,
    searcher_client: &mut SearcherClient,
    token_result: &mut TokenResult,
) -> Result<(), Box<dyn Error>> {
    let mint = if new_pool_info.input_mint.to_string()
        == constants::SOLANA_PROGRAM_ID
    {
        new_pool_info.output_mint
    } else {
        new_pool_info.input_mint
    };
    token_result.mint = mint.to_string();
    token_result.amm_pool = new_pool_info.amm_pool_id.to_string();

    info!("processing {}", mint.to_string());

    let pubsub_client = PubsubClient::new(dotenv!("WS_URL"))
        .await
        .expect("pubsub client async");

    let (sol_pooled, ok) = listen_for_sol_pooled(
        &new_pool_info.amm_pool_id,
        &provider.rpc_client,
        &pubsub_client,
    )
    .await?;
    token_result.sol_pooled = Some(sol_pooled);
    if !ok {
        token_result.outcome = "insufficient sol pooled".to_string();
        return Ok(());
    }

    let (burn_pct, ok) = listen_for_burn(
        &new_pool_info.amm_pool_id,
        &provider.rpc_client,
        &pubsub_client,
    )
    .await?;
    if !ok {
        if burn_pct == -1. {
            token_result.outcome =
                "rug pull (insufficient sol pooled post-LP-action)".to_string();
        } else {
            token_result.outcome = "insufficient burn pct".to_string();
            token_result.burn_pct = Some(burn_pct);
        }
        return Ok(());
    }

    let (top_10_holders, ok) = check_top_holders(&mint, provider).await?;
    token_result.top_10_holders = Some(top_10_holders);
    if !ok {
        token_result.outcome = "centralized supply".to_string();
        return Ok(());
    }

    // this should be converted to listening as well
    // some tokens have mint and freeze authority disabled a bit later
    // but mostly before the burn
    let (is_safe, msg) =
        provider.sanity_check(&mint).await.expect("sanity check");
    if !is_safe {
        warn!("{} Unsafe pool: {}, skipping", mint, msg);
        token_result.outcome = msg;
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
    let quick = false;
    let mut ixs =
        raydium::make_swap_ixs(provider, wallet, &swap_context, quick)
            .await
            .expect("make swap ixs");

    info!("took {:?} to pack", start.elapsed());

    let tip = 50000;

    let swap_result = jito::send_swap_tx(
        &mut ixs,
        tip,
        wallet,
        searcher_client,
        &provider.rpc_client,
    )
    .await;

    token_result.timestamp_finalized =
        chrono::Utc::now().timestamp().to_string();

    token_result.outcome = "success (if bundle landed)".to_string();

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

// pub async fn append_result(
//     file_path: &str,
//     record: TokenResult,
// ) -> Result<(), Box<dyn Error>> {
//     // Open the file in append mode. Create it if it does not exist.
//     let file = OpenOptions::new()
//         .write(true)
//         .append(true)
//         .create(true)
//         .open(file_path)
//         .await?;
//
//     let mut writer = Writer::from_writer(file);
//
//     // Check if the file is empty to avoid writing headers multiple times
//     if writer.has_headers() {
//         writer.write_record(&["Field1", "Field2", "Field3"])?;
//     }
//
//     // Serialize and append the record
//     writer.serialize(record)?;
//
//     // Flush to ensure all writes are committed
//     writer.flush()?;
//     Ok(())
// }
