use std::error::Error;

use base64::Engine;
use futures_util::StreamExt;
use log::{debug, info, warn};
use raydium_amm::{math::SwapDirection, state::AmmInfo};
use raydium_library::amm;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::constants;

#[derive(Debug, Default)]
pub struct VaultState {
    pub slot: u64,
    pub amount: u64,
    pub decimals: u8,
}

#[derive(Debug, Default)]
pub struct Pool {
    pub token_vault: VaultState,
    pub sol_vault: VaultState,
    pub token_mint: Pubkey,
}

impl Pool {
    pub fn try_price(&self) -> Option<f64> {
        if self.token_vault.amount == 0
            || self.sol_vault.amount == 0
            || self.sol_vault.slot == 0
            || self.token_vault.slot == 0
            || self.sol_vault.slot != self.token_vault.slot
        {
            return None;
        }
        // decimals hard-coded as 6 (most-common), might lead to weird errors,
        // worth pulling it from chain, same as SOL price, this method is more
        // for looking, for trading another method should be used that returns the ratio
        // ratio is all
        let token_amount = self.token_vault.amount as f64
            / 10u64.pow(self.token_vault.decimals as u32) as f64;
        let sol_amount = self.sol_vault.amount as f64 / 10u64.pow(9) as f64;
        Some(sol_amount / token_amount * 170.)
    }

    pub fn calculate_token_amount_out(&self, lamports_in: u64) -> u64 {
        raydium_amm::math::Calculator::swap_token_amount_base_in(
            lamports_in.into(),
            self.token_vault.amount.into(),
            self.sol_vault.amount.into(),
            SwapDirection::PC2Coin,
        )
        .as_u64()
    }

    pub fn calculate_sol_amount_out(&self, token_in: u64) -> u64 {
        // for some reason the decimals are not taken into account in the spl_state
        raydium_amm::math::Calculator::swap_token_amount_base_in(
            token_in
                // .checked_mul(10u64.pow(self.token_vault.decimals as u32))
                // .expect("mul by decimals")
                .into(),
            self.token_vault.amount.into(),
            self.sol_vault.amount.into(),
            SwapDirection::PC2Coin,
        )
        .as_u64()
    }
}

pub async fn listen_price(
    amm_pool: &Pubkey,
    rpc_client: &RpcClient,
    pubsub_client: &PubsubClient,
) -> Result<bool, Box<dyn Error>> {
    // load amm keys
    let amm_program = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
    let amm_keys =
        amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    let coin_mint_is_sol =
        amm_keys.amm_coin_mint.eq(&constants::SOLANA_PROGRAM_ID);
    let (token_vault, sol_vault) = if coin_mint_is_sol {
        (amm_keys.amm_pc_vault, amm_keys.amm_coin_vault)
    } else {
        (amm_keys.amm_coin_vault, amm_keys.amm_pc_vault)
    };
    let token_mint = if coin_mint_is_sol {
        amm_keys.amm_pc_mint
    } else {
        amm_keys.amm_coin_mint
    };

    let (mut token_stream, token_unsub) = pubsub_client
        .account_subscribe(
            &token_vault,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| {
            Box::<dyn Error>::from(format!(
                "Failed to subscribe to token account: {}",
                e
            ))
        })?;

    let (mut sol_stream, sol_unsub) = pubsub_client
        .account_subscribe(
            &sol_vault,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| {
            Box::<dyn Error>::from(format!(
                "Failed to subscribe to SOL account: {}",
                e
            ))
        })?;

    let mut pool = Pool::default();
    info!("listening for price for {}", token_mint.to_string());
    loop {
        tokio::select! {
            Some(token_log) = token_stream.next() => {
                match token_log.value.data {
                    UiAccountData::Binary(data, UiAccountEncoding::Base64) => {
                        let Ok(log_data) = base64::prelude::BASE64_STANDARD.decode(data) else {
                            warn!("decode token b64");
                            continue;
                        };
                        if log_data.is_empty() {
                            warn!("empty log data");
                            continue;
                        }
                        let Ok(account) = spl_token::state::Account::unpack(&log_data) else {
                            warn!("unpack token account");
                            continue;
                        };
                        pool.token_vault.amount = account.amount;
                        pool.token_vault.slot = token_log.context.slot;
                        if let Some(price) = pool.try_price() {
                            info!("price: {}", price);
                        }
                    }
                    _ => {
                        warn!("unexpected data");
                    }
                }
            }
            Some(sol_log) = sol_stream.next() => {
                pool.sol_vault.amount = sol_log.value.lamports;
                pool.sol_vault.slot = sol_log.context.slot;
                if let Some(price) = pool.try_price() {
                    info!("price: {}", price);
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(3000)) => {
                warn!("timeout");
                token_unsub().await;
                sol_unsub().await;
                return Ok(true);
            }
        }
    }
}

pub fn clear() {
    print!("\x1b[2J\x1b[1;1H");
}

pub async fn get_sol_pooled_vault(
    vault: &Pubkey,
    rpc_client: &RpcClient,
) -> f64 {
    let sol_pooled = rpc_client.get_account(vault).await.unwrap().lamports;
    sol_pooled as f64 / 10u64.pow(9) as f64
}

pub async fn get_sol_pooled(amm_pool: &Pubkey, rpc_client: &RpcClient) -> f64 {
    let amm_info = unpack::<AmmInfo>(
        &rpc_client
            .get_account_data(amm_pool)
            .await
            .expect("get amm pool"),
    )
    .expect("unpack");
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
            if amm_info.coin_vault_mint.eq(&constants::SOLANA_PROGRAM_ID) {
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

pub async fn get_decimals(mint: &Pubkey, rpc_client: &RpcClient) -> u8 {
    let mint_account = rpc_client
        .get_account(mint)
        .await
        .expect("get mint account");
    let mint_data =
        Mint::unpack(&mint_account.data).expect("unpack mint data");
    mint_data.decimals
}

pub async fn get_spl_balance(
    rpc_client: &RpcClient,
    token_account: &Pubkey,
) -> Result<u64, Box<dyn std::error::Error>> {
    let mut backoff = 100;
    for _ in 0..12 {
        match rpc_client.get_token_account_balance(token_account).await {
            Ok(balance) => {
                if balance.amount == "0" {
                    continue;
                }
                return match balance.amount.parse::<u64>() {
                    Ok(parsed_balance) => Ok(parsed_balance),
                    Err(e) => Err(Box::new(e)),
                };
            }
            Err(e) => {
                warn!(
                    "{} error getting balance: {}",
                    token_account.to_string(),
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    backoff,
                ))
                .await;
                backoff *= 2;
                continue;
            }
        };
    }
    Err(format!("could not fetch balance for {}", token_account).into())
}

pub async fn get_spl_balance_stream(
    pubsub_client: &PubsubClient,
    token_account: &Pubkey,
) -> Result<u64, Box<dyn std::error::Error>> {
    let Ok((mut stream, unsub)) = pubsub_client
        .account_subscribe(
            token_account,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await
    else {
        warn!(
            "get_spl_balance_stream {}: subscribe",
            token_account.to_string()
        );
        return Err("subscribe".into());
    };

    tokio::select! {
        log = stream.next() => {
            if let UiAccountData::Binary(data, UiAccountEncoding::Base64) = log.expect("log").value.data {
                let Ok(log_data) = base64::prelude::BASE64_STANDARD.decode(&data) else {
                    warn!("decode spl b64");
                    return Err("decode spl b64".into());
                };
                let Ok(spl_account) = spl_token::state::Account::unpack(&log_data) else {
                    warn!("unpack spl account");
                    return Err("unpack spl account".into());
                };
                unsub().await;
                Ok(spl_account.amount)
            } else {
                warn!("get_spl_balance_stream {}: unexpected data", token_account.to_string());
                Err("unexpected data".into())
            }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
            warn!("get_spl_balance_stream {}: timeout", token_account.to_string());
            Err("timeout".into())
        },
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;

    use crate::util::env;

    use super::{unpack, AmmInfo};

    #[tokio::test]
    async fn test_get_decimals() {
        let mint =
            Pubkey::from_str("6hm9tDfhnhVCBD6Qk8L27WabnbzfUJFs5jQpdLnNVAET")
                .unwrap();
        let decimals =
            super::get_decimals(&mint, &RpcClient::new(env("RPC_URL"))).await;
        assert!(decimals == 5u8);
    }

    #[test]
    fn test_parse_amm() {
        let raw_data = vec![
            6, 0, 0, 0, 0, 0, 0, 0, 254, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            64, 122, 16, 243, 90, 0, 0, 244, 1, 0, 0, 0, 0, 0, 0, 128, 240,
            250, 2, 0, 0, 0, 0, 0, 228, 11, 84, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 202, 154, 59, 0, 0, 0, 0, 0, 228,
            11, 84, 2, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 16, 39, 0, 0, 0, 0, 0,
            0, 25, 0, 0, 0, 0, 0, 0, 0, 16, 39, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0,
            0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 16,
            39, 0, 0, 0, 0, 0, 0, 90, 101, 13, 61, 39, 1, 0, 0, 95, 206, 74,
            109, 21, 0, 0, 0, 85, 68, 118, 37, 114, 21, 0, 0, 21, 2, 178, 255,
            7, 89, 2, 0, 159, 166, 242, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 233, 185,
            73, 9, 228, 142, 3, 0, 0, 0, 0, 0, 0, 0, 0, 212, 72, 45, 251, 67,
            10, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 11, 218, 9, 143, 21, 0, 0,
            54, 52, 25, 96, 127, 175, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 102, 240,
            102, 11, 61, 106, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 120, 63, 224, 87,
            248, 70, 2, 0, 210, 186, 131, 72, 188, 117, 137, 37, 83, 54, 141,
            182, 89, 240, 190, 65, 16, 155, 72, 164, 133, 239, 20, 188, 142,
            204, 13, 233, 232, 114, 223, 190, 236, 83, 200, 41, 49, 116, 148,
            39, 108, 82, 195, 123, 202, 5, 98, 83, 21, 3, 218, 188, 24, 148,
            171, 54, 6, 229, 29, 247, 127, 48, 193, 80, 13, 131, 35, 192, 118,
            240, 226, 135, 24, 202, 96, 215, 126, 107, 57, 206, 232, 242, 63,
            67, 207, 196, 255, 31, 88, 82, 184, 252, 27, 148, 162, 147, 6,
            155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192,
            53, 218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0,
            1, 104, 166, 127, 248, 190, 13, 37, 30, 185, 46, 15, 154, 34, 229,
            78, 229, 209, 70, 42, 187, 55, 69, 155, 90, 163, 68, 21, 203, 153,
            183, 74, 128, 31, 183, 164, 72, 200, 185, 198, 154, 26, 166, 158,
            44, 45, 221, 155, 92, 70, 157, 95, 169, 55, 38, 95, 180, 126, 243,
            129, 21, 18, 114, 23, 151, 61, 114, 173, 22, 168, 26, 187, 204,
            142, 58, 153, 54, 64, 140, 32, 82, 75, 48, 235, 38, 159, 107, 111,
            64, 146, 120, 196, 151, 137, 214, 115, 6, 13, 7, 81, 168, 40, 45,
            166, 19, 5, 254, 41, 156, 55, 185, 152, 229, 132, 113, 219, 17,
            53, 3, 115, 16, 248, 190, 16, 69, 166, 10, 246, 238, 201, 231,
            199, 78, 24, 154, 162, 200, 203, 71, 89, 101, 110, 68, 233, 71,
            239, 121, 89, 102, 184, 220, 54, 44, 170, 16, 48, 56, 248, 250,
            49, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 229, 182, 43, 101, 203, 59, 189, 166, 245, 104, 136, 230, 111,
            238, 142, 100, 220, 85, 96, 25, 156, 15, 136, 177, 31, 226, 115,
            189, 5, 158, 138, 161, 135, 145, 245, 195, 129, 26, 2, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let amm_info = unpack::<AmmInfo>(&raw_data).unwrap();
        assert!(
            amm_info.lp_mint.to_string()
                == "83WevmL2JzaEvDmuJUFMxcFNnHqP4xonfvAzKmsPWjwu"
        );
        assert!(
            amm_info.open_orders.to_string()
                == "38p42yoKFWgxw2LCbB96wAKa2LwAxiBArY3fc3eA9yWv"
        );
        assert!(
            amm_info.market_program.to_string()
                == "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"
        );
    }

    #[test]
    fn test_get_ata_pump() {
        let mint = "FAJVRnNHuwozDi5UL8guMyobveadXDFxeikvN4Hupump".to_string();
        let expected_addr =
            "HKh1cnq5b5iuhcEiDNyyGFyw3877hLzUjGCgr4LFjfHC".to_string();
        let ata = spl_associated_token_account::get_associated_token_address(
            &Pubkey::from_str("fuckTYubuBRLPm3TnBWfYYkDKnnfJqRtk1L1DpE4uFK")
                .unwrap(),
            &Pubkey::from_str(&mint).unwrap(),
        );
        assert!(ata.to_string() == expected_addr);
    }
}
