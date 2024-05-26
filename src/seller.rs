use std::{error::Error, str::FromStr};

use base64::Engine;
use futures_util::StreamExt;
use log::{debug, info};
use raydium_amm::state::AmmInfo;
use raydium_library::amm;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
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
        Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY).expect("amm program");
    let amm_keys = amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    let coin_mint_is_sol =
        amm_keys.amm_coin_mint == Pubkey::from_str(constants::SOLANA_PROGRAM_ID).expect("sol mint");

    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            amm_pool,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
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
        match log.value.data {
            UiAccountData::Binary(data, UiAccountEncoding::Base64) => {
                let _ = unpack::<AmmInfo>(&base64::prelude::BASE64_STANDARD.decode(data).unwrap())
                    .expect("unpack amm info");
                // get_sol_pooled(&amm_info, rpc_client).await;
            }
            _ => {
                info!("unexpected data format, only base64 for now");
            }
        }
    }

    unsub().await;

    Ok(0.)
}

pub fn clear() {
    print!("\x1b[2J\x1b[1;1H");
}

pub async fn get_sol_pooled(amm_pool: &Pubkey, rpc_client: &RpcClient) -> f64 {
    let amm_info = unpack::<AmmInfo>(
        &rpc_client
            .get_account_data(amm_pool)
            .await
            .expect("get amm pool"),
    )
    .expect("unpack");
    // clear();
    debug!("market {}", amm_info.market.to_string());
    debug!("pc in {}", amm_info.state_data.swap_pc_in_amount);
    debug!("pc out {}", amm_info.state_data.swap_pc_out_amount);
    debug!("coin in {}", amm_info.state_data.swap_coin_in_amount);
    debug!("coin out {}", amm_info.state_data.swap_coin_out_amount);
    debug!("pc vault {}", amm_info.pc_vault);
    debug!("coint vault {}", amm_info.coin_vault);
    debug!("lp amount {}", amm_info.lp_amount);
    // check the diff between in and out

    let sol_pooled = rpc_client
        .get_token_account_balance(
            if amm_info.coin_vault_mint.to_string() == constants::SOLANA_PROGRAM_ID {
                &amm_info.coin_vault
            } else {
                &amm_info.pc_vault
            },
        )
        .await
        .expect("sol pooled");

    sol_pooled.ui_amount.unwrap()
}

pub fn unpack<T>(data: &[u8]) -> Option<T>
where
    T: Clone,
{
    let ret = unsafe { &*(&data[0] as *const u8 as *const T) };
    Some(ret.clone())
}

#[cfg(test)]
mod tests {

    use super::{unpack, AmmInfo};

    #[test]
    fn test_parse_amm() {
        let raw_data = vec![
            6, 0, 0, 0, 0, 0, 0, 0, 254, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 64, 122, 16, 243, 90, 0, 0, 244, 1, 0, 0, 0, 0, 0, 0, 128, 240,
            250, 2, 0, 0, 0, 0, 0, 228, 11, 84, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 202, 154, 59, 0, 0, 0, 0, 0, 228, 11, 84, 2, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0,
            0, 16, 39, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 16, 39, 0, 0, 0, 0, 0, 0, 12, 0,
            0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 16, 39, 0, 0, 0,
            0, 0, 0, 90, 101, 13, 61, 39, 1, 0, 0, 95, 206, 74, 109, 21, 0, 0, 0, 85, 68, 118, 37,
            114, 21, 0, 0, 21, 2, 178, 255, 7, 89, 2, 0, 159, 166, 242, 101, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 233, 185, 73, 9,
            228, 142, 3, 0, 0, 0, 0, 0, 0, 0, 0, 212, 72, 45, 251, 67, 10, 33, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 224, 11, 218, 9, 143, 21, 0, 0, 54, 52, 25, 96, 127, 175, 33, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 102, 240, 102, 11, 61, 106, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 120, 63, 224, 87,
            248, 70, 2, 0, 210, 186, 131, 72, 188, 117, 137, 37, 83, 54, 141, 182, 89, 240, 190,
            65, 16, 155, 72, 164, 133, 239, 20, 188, 142, 204, 13, 233, 232, 114, 223, 190, 236,
            83, 200, 41, 49, 116, 148, 39, 108, 82, 195, 123, 202, 5, 98, 83, 21, 3, 218, 188, 24,
            148, 171, 54, 6, 229, 29, 247, 127, 48, 193, 80, 13, 131, 35, 192, 118, 240, 226, 135,
            24, 202, 96, 215, 126, 107, 57, 206, 232, 242, 63, 67, 207, 196, 255, 31, 88, 82, 184,
            252, 27, 148, 162, 147, 6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24,
            192, 53, 218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1, 104, 166,
            127, 248, 190, 13, 37, 30, 185, 46, 15, 154, 34, 229, 78, 229, 209, 70, 42, 187, 55,
            69, 155, 90, 163, 68, 21, 203, 153, 183, 74, 128, 31, 183, 164, 72, 200, 185, 198, 154,
            26, 166, 158, 44, 45, 221, 155, 92, 70, 157, 95, 169, 55, 38, 95, 180, 126, 243, 129,
            21, 18, 114, 23, 151, 61, 114, 173, 22, 168, 26, 187, 204, 142, 58, 153, 54, 64, 140,
            32, 82, 75, 48, 235, 38, 159, 107, 111, 64, 146, 120, 196, 151, 137, 214, 115, 6, 13,
            7, 81, 168, 40, 45, 166, 19, 5, 254, 41, 156, 55, 185, 152, 229, 132, 113, 219, 17, 53,
            3, 115, 16, 248, 190, 16, 69, 166, 10, 246, 238, 201, 231, 199, 78, 24, 154, 162, 200,
            203, 71, 89, 101, 110, 68, 233, 71, 239, 121, 89, 102, 184, 220, 54, 44, 170, 16, 48,
            56, 248, 250, 49, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 229, 182, 43, 101, 203, 59, 189, 166, 245, 104,
            136, 230, 111, 238, 142, 100, 220, 85, 96, 25, 156, 15, 136, 177, 31, 226, 115, 189, 5,
            158, 138, 161, 135, 145, 245, 195, 129, 26, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let amm_info = unpack::<AmmInfo>(&raw_data).unwrap();
        assert!(amm_info.lp_mint.to_string() == "83WevmL2JzaEvDmuJUFMxcFNnHqP4xonfvAzKmsPWjwu");
        assert!(amm_info.open_orders.to_string() == "38p42yoKFWgxw2LCbB96wAKa2LwAxiBArY3fc3eA9yWv");
        assert!(
            amm_info.market_program.to_string() == "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"
        );
    }
}
