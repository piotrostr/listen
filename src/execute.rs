use std::str::FromStr;

use base64::Engine;
use futures_util::StreamExt;
use log::{info, warn};
use raydium_library::amm;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::{
    nonblocking::{
        pubsub_client::{self, PubsubClient},
        rpc_client::RpcClient,
    },
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey, signature::Keypair,
};

use crate::{buyer, constants, jito, seller::Pool, Provider};

#[derive(Debug)]
pub struct Executor {
    pub lamports_in: u64,
    pub token_balance: u64,
    pub funder: Keypair,

    pub amm_keys: amm::AmmKeys,

    // denoted as pct, bool flag vec
    pub tp_levels: Vec<f64>,
    pub tp_amounts: Vec<f64>,
    pub tp_reached: Vec<bool>,

    pub sl_levels: Vec<f64>,
    pub sl_amounts: Vec<f64>,
    pub sl_reached: Vec<bool>,
}

impl Executor {
    pub async fn execute(
        &mut self,
        provider: &Provider,
        pubsub_client: &PubsubClient,
        amm_pool: &Pubkey,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let coin_mint_is_sol = self.amm_keys.amm_coin_mint
            == Pubkey::from_str(constants::SOLANA_PROGRAM_ID).expect("sol mint");
        let (token_vault, sol_vault) = if coin_mint_is_sol {
            (self.amm_keys.amm_pc_vault, self.amm_keys.amm_coin_vault)
        } else {
            (self.amm_keys.amm_coin_vault, self.amm_keys.amm_pc_vault)
        };
        let token_mint = if coin_mint_is_sol {
            self.amm_keys.amm_pc_mint
        } else {
            self.amm_keys.amm_coin_mint
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
            .expect("subscribe to account");

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
            .expect("subscribe to account");

        let mut pool = Pool::default();

        info!("listening for price for {}", token_mint.to_string());
        loop {
            tokio::select! {
                Some(token_log) = token_stream.next() => {
                    match token_log.value.data {
                        UiAccountData::Binary(data, UiAccountEncoding::Base64) => {
                            let log_data = base64::prelude::BASE64_STANDARD.decode(data).unwrap();
                            if log_data.is_empty() {
                                warn!("empty log data");
                                continue;
                            }
                            let account = spl_token::state::Account::unpack(&log_data).unwrap();
                            pool.token_vault.amount = account.amount;
                            pool.token_vault.slot = token_log.context.slot;
                            if pool.try_price().is_some() {
                                let lamports_out = pool.calculate_sol_amount_out(self.token_balance);
                                let sell_amount = self.get_sell_amount(self.lamports_in, lamports_out);
                                if sell_amount != 0 {
                                    buyer::swap(
                                        amm_pool,
                                        &token_mint,
                                        &Pubkey::from_str(constants::SOLANA_PROGRAM_ID).unwrap(),
                                        sell_amount,
                                        &self.funder,
                                        &provider
                                    ).await.expect("swap");
                                }
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
                    if pool.try_price().is_some() {
                        let lamports_out = pool.calculate_sol_amount_out(self.token_balance);
                        let sell_amount = self.get_sell_amount(self.lamports_in, lamports_out);
                        if sell_amount != 0 {
                            buyer::swap(
                                amm_pool,
                                &token_mint,
                                &Pubkey::from_str(constants::SOLANA_PROGRAM_ID).unwrap(),
                                sell_amount,
                                &self.funder,
                                &provider
                            ).await.expect("swap");
                        }
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

    pub fn get_sell_amount(&mut self, lamports_in: u64, lamports_out: u64) -> u64 {
        let mut sell_amount = 0.;
        let lamports_in = lamports_in as f64;
        let lamports_out = lamports_out as f64;
        for i in 0..self.tp_levels.len() {
            if *self.tp_reached.get(i).unwrap() {
                continue;
            }
            let tp_level = self.tp_levels.get(i).unwrap();
            let tp_amount = self.tp_amounts.get(i).unwrap();
            if lamports_out >= *tp_level as f64 * lamports_in {
                sell_amount += tp_amount;
                self.tp_reached[i] = true;
            }
        }
        for i in 0..self.sl_levels.len() {
            let sl_level = self.sl_levels.get(i).unwrap();
            if *self.sl_reached.get(i).unwrap() {
                continue;
            }
            let sl_amount = self.sl_amounts.get(i).unwrap();
            if lamports_out <= *sl_level as f64 * lamports_in {
                sell_amount += sl_amount;
                self.sl_reached[i] = true;
            }
        }
        sell_amount as u64
    }
}
