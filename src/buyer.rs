use std::{error::Error, str::FromStr, sync::Arc};

use crate::{
    checker::Checklist,
    constants, jito,
    provider::Provider,
    raydium::{self, get_burn_pct},
    util::env,
};
use futures_util::StreamExt;
use jito_searcher_client::get_searcher_client;
use log::{debug, info, warn};
use raydium_library::amm;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair, signer::EncodableKey,
};
use spl_token::state::Mint;

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct TokenResult {
    pub creation_signature: String,
    pub timestamp_received: String,
    pub timestamp_finalized: String,
    pub checklist: Checklist,
}

pub async fn buy(
    amm_pool: &Pubkey,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
    wallet: &Keypair,
    provider: &Provider,
) -> Result<(), Box<dyn Error>> {
    let swap_context = raydium::make_swap_context(
        provider,
        *amm_pool,
        *input_mint,
        *output_mint,
        wallet,
        0,
        amount,
    )
    .await
    .expect("makes swap context");

    let start = std::time::Instant::now();
    let quick = true;
    let mut ixs =
        raydium::make_swap_ixs(provider, wallet, &swap_context, quick)
            .await
            .expect("make swap ixs");

    info!("took {:?} to pack", start.elapsed());

    let tip = 50000;
    let auth = Keypair::read_from_file(env("AUTH_KEYPAIR_PATH"))
        .expect("read auth keypair");
    let mut searcher_client =
        get_searcher_client(&env("BLOCK_ENGINE_URL"), &Arc::new(auth))
            .await
            .expect("makes searcher client");

    let swap_result = jito::send_swap_tx(
        &mut ixs,
        tip,
        wallet,
        &mut searcher_client,
        &provider.rpc_client,
    )
    .await;

    info!("{:?}", swap_result);

    Ok(())
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
    if top_10_holders > 0.35 {
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
    if stream.next().await.is_some() {
        let (result, _, amm_keys) =
            raydium::get_calc_result(rpc_client, amm_pool).await?;
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
        if sol_pooled >= 30. {
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
                commitment: Some(CommitmentConfig::processed()),
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

pub async fn check_if_pump_fun(mint: &Pubkey) -> Result<bool, Box<dyn Error>> {
    let base = "https://client-api-2-74b1891ee9f9.herokuapp.com/coins/";
    let url = format!("{}{}", base, mint);
    let res = reqwest::get(&url).await?;
    Ok(res.status().is_success())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_sdk::pubkey::Pubkey;

    #[tokio::test]
    async fn test_check_if_pump_fun_works_for_pump_fun() {
        // some pump fun shitto
        let mint =
            Pubkey::from_str("2yqz8eJvJu1eiaYz34r9i7YbyTveRRJwPFhRJenp6yed")
                .unwrap();
        let res = super::check_if_pump_fun(&mint).await.unwrap();
        assert!(res == true);
    }

    #[tokio::test]
    async fn test_check_if_pump_fun_works_for_not_pump_fun() {
        // wifhat
        let mint =
            Pubkey::from_str("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm")
                .unwrap();
        let res = super::check_if_pump_fun(&mint).await.unwrap();
        assert!(res == false);
    }
}
