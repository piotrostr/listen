use std::{
    error::Error, str::FromStr, sync::Arc, thread::sleep, time::Duration,
};

use crate::{
    constants, jito,
    raydium::{self, get_burn_pct},
    util::env,
};
use futures_util::StreamExt;
use jito_searcher_client::get_searcher_client;
use log::{debug, info, warn};
use raydium_library::amm;
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

pub async fn swap(
    amm_pool: &Pubkey,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
    wallet: &Keypair,
    rpc_client: &RpcClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut retries = 0;
    let mut backoff = 100u64;
    let swap_context = loop {
        match raydium::make_swap_context(
            rpc_client,
            *amm_pool,
            *input_mint,
            *output_mint,
            wallet,
            0,
            amount,
        )
        .await
        {
            Ok(swap_context) => {
                break Some(swap_context);
            }
            Err(_) => {
                warn!("make swap context failed");
                sleep(Duration::from_millis(backoff));
                if retries > 6 {
                    break None;
                }
                backoff *= 2;
                retries += 1;
            }
        }
    };
    let Some(swap_context) = swap_context else {
        return Err("make swap context failed".into());
    };

    let start = std::time::Instant::now();
    let quick = true;
    let Ok(mut ixs) =
        raydium::make_swap_ixs(rpc_client, wallet, &swap_context, quick).await
    else {
        return Err("make swap ixs".into());
    };

    info!("took {:?} to pack", start.elapsed());

    info!("swapping {} {} to {}", amount, input_mint, output_mint);
    let Ok(auth) = Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")) else {
        return Err("read auth kp".into());
    };

    let Ok(mut searcher_client) =
        get_searcher_client(&env("BLOCK_ENGINE_URL"), &Arc::new(auth)).await
    else {
        return Err("makes searcher client".into());
    };

    if let Err(e) = jito::send_swap_tx_no_wait(
        &mut ixs,
        50000,
        wallet,
        &mut searcher_client,
        rpc_client,
    )
    .await
    {
        return Err(format!("send swap tx (jito) {}", e).into());
    }

    drop(searcher_client);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum TopHoldersCheckError {
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
}

pub async fn check_top_holders(
    mint: &Pubkey,
    rpc_client: &RpcClient,
    string_output: bool,
) -> Result<(f64, bool, String), TopHoldersCheckError> {
    let top_holders = rpc_client
        .get_token_largest_accounts(mint)
        .await
        .map_err(|e| TopHoldersCheckError::RpcError(e.to_string()))?;

    let up_to_ten = 10.min(top_holders.len());
    let top_holders = top_holders[0..up_to_ten].to_vec();
    let top_holders_len = top_holders.len();

    let total_supply = rpc_client
        .get_token_supply(mint)
        .await
        .map_err(|e| TopHoldersCheckError::RpcError(e.to_string()))?
        .ui_amount
        .ok_or_else(|| {
            TopHoldersCheckError::InvalidAccount("No ui_amount".to_string())
        })?;

    let mut total = 0f64;
    let mut got_raydium = false;
    let mut raydium_holding = 0f64;

    let res = top_holders.clone();

    for holder in top_holders {
        debug!("holder: {:?}, balance: {:?}", holder.address, holder.amount);
        if !got_raydium {
            let account_info = rpc_client
                .get_token_account_with_commitment(
                    &Pubkey::from_str(holder.address.as_str()).map_err(
                        |e| TopHoldersCheckError::ParseError(e.to_string()),
                    )?,
                    CommitmentConfig::processed(),
                )
                .await
                .map_err(|e| TopHoldersCheckError::RpcError(e.to_string()))?;

            if account_info
                .value
                .ok_or_else(|| {
                    TopHoldersCheckError::InvalidAccount(
                        "No account info".to_string(),
                    )
                })?
                .owner
                == constants::RAYDIUM_AUTHORITY_V4_PUBKEY.to_string()
            {
                raydium_holding =
                    holder.amount.ui_amount.ok_or_else(|| {
                        TopHoldersCheckError::InvalidAccount(
                            "No ui_amount".to_string(),
                        )
                    })?;
                got_raydium = true;
            }
        }
        total += holder.amount.ui_amount.ok_or_else(|| {
            TopHoldersCheckError::InvalidAccount("No ui_amount".to_string())
        })?;
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
        return Ok((top_10_holders, false, "".to_string()));
    }

    if string_output {
        return Ok((
            top_10_holders,
            true,
            res.iter()
                .map(|holder| {
                    format!(
                        "{}: {}",
                        holder.address,
                        holder.amount.ui_amount.unwrap()
                    )
                })
                .collect::<Vec<String>>()
                .join(", "),
        ));
    }

    Ok((top_10_holders, true, "".to_string()))
}

pub async fn listen_for_sol_pooled(
    amm_pool: &Pubkey,
    rpc_client: &RpcClient,
    pubsub_client: &PubsubClient,
) -> Result<(f64, bool), Box<dyn Error>> {
    let Ok((mut stream, unsub)) = pubsub_client
        .account_subscribe(
            amm_pool,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                ..Default::default()
            }),
        )
        .await
    else {
        return Err("subscribe to account".into());
    };

    info!("listening for sol pooled for pool {}", amm_pool.to_string());
    if stream.next().await.is_some() {
        let (result, _, amm_keys) =
            raydium::get_calc_result(rpc_client, amm_pool).await?;
        let coin_mint_is_sol =
            amm_keys.amm_coin_mint.eq(&constants::SOLANA_PROGRAM_ID);
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
    let amm_program = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
    let amm_keys =
        amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    let lp_mint = amm_keys.amm_lp_mint;
    let coin_mint_is_sol =
        amm_keys.amm_coin_mint.eq(&constants::SOLANA_PROGRAM_ID);

    let Ok((mut stream, unsub)) = pubsub_client
        .account_subscribe(
            &lp_mint,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                ..Default::default()
            }),
        )
        .await
    else {
        return Err("subscribe to account".into());
    };

    let token_mint = if coin_mint_is_sol {
        amm_keys.amm_pc_mint
    } else {
        amm_keys.amm_coin_mint
    };

    info!("listening for burn for {}", token_mint.to_string());
    while let Some(log) = stream.next().await {
        debug!("log: {:?}", log);
        if let UiAccountData::LegacyBinary(data) = log.value.data {
            let Ok(mint_data) =
                Mint::unpack(bs58::decode(data).into_vec()?.as_slice())
            else {
                return Err("unpack mint data".into());
            };
            debug!("mint data: {:?}", mint_data);

            let (result, _, _) =
                raydium::get_calc_result(rpc_client, amm_pool).await?;

            // check if any sol pooled before checking burn_pct for correct res
            // rug-pulled tokens have LP supply of 0
            let sol_pooled = raydium::calc_result_to_financials(
                coin_mint_is_sol,
                result,
                0,
            );
            if sol_pooled < 1. {
                warn!("{} rug pull, sol pooled: {}", token_mint, sol_pooled);
                return Ok((-1., false));
            }

            let Ok(burn_pct) = get_burn_pct(mint_data, result) else {
                return Err("get burn pct".into());
            };
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
    // easier way, also skips bundled tokens without the derived "pump" suffix
    if mint.to_string().ends_with("pump") {
        Ok(true)
    } else {
        // let base = "https://frontend-api.pump.fun/coins/";
        // let url = format!("{}{}", base, mint);
        // let res = reqwest::get(&url).await?;
        // Ok(res.status().is_success())
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_sdk::pubkey::Pubkey;

    #[tokio::test]
    async fn test_check_if_pump_fun_works_for_pump_fun() {
        // some pump fun shitto
        let mint =
            Pubkey::from_str("FAJVRnNHuwozDi5UL8guMyobveadXDFxeikvN4Hupump")
                .unwrap();
        let res = super::check_if_pump_fun(&mint).await.unwrap();
        assert!(res);
    }

    #[tokio::test]
    async fn test_check_if_pump_fun_works_for_not_pump_fun() {
        // wifhat
        let mint =
            Pubkey::from_str("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm")
                .unwrap();
        let res = super::check_if_pump_fun(&mint).await.unwrap();
        assert!(!res);
    }
}
