use std::{error::Error, str::FromStr};

use futures_util::StreamExt;
use log::info;
use raydium_amm::state::AmmInfo;
use raydium_library::amm;
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use crate::constants;

pub async fn listen_price(
    amm_pool: &Pubkey,
    rpc_client: &RpcClient,
    pubsub_client: &PubsubClient,
) -> Result<f64, Box<dyn Error>> {
    // load amm keys
    let amm_program =
        Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)
            .expect("amm program");
    let amm_keys =
        amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    let coin_mint_is_sol = amm_keys.amm_coin_mint
        == Pubkey::from_str(constants::SOLANA_PROGRAM_ID).expect("sol mint");

    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            &amm_pool,
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

    info!("listening for price for {}", token_mint.to_string());
    while let Some(log) = stream.next().await {
        if let UiAccountData::LegacyBinary(data) = log.value.data {
            let amm_info =
                unpack::<AmmInfo>(&data.as_bytes()).expect("unpack amm info");
            info!("amm_data: {:?}", amm_info);
        }
    }

    unsub().await;

    Ok(0.)
}

pub fn unpack<T>(data: &[u8]) -> Option<T>
where
    T: Clone,
{
    let ret = unsafe { &*(&data[0] as *const u8 as *const T) };
    Some(ret.clone())
}
