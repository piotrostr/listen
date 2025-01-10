use std::str::FromStr;

use base64::Engine;
use futures_util::StreamExt;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcAccountInfoConfig, RpcTransactionConfig},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction, UiParsedMessage,
    UiPartiallyDecodedInstruction, UiTransactionEncoding,
};
use spl_token::state::Mint;

use crate::{
    buyer::check_if_pump_fun,
    constants,
    util::{env, pubkey_to_string, string_to_pubkey},
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Checklist {
    pub slot: u64,
    pub is_pump_fun: bool,
    pub lp_burnt: bool,
    pub mint_authority_renounced: bool,
    pub freeze_authority_renounced: bool,
    pub sol_pooled: f64,
    pub timeout: bool,
    pub accounts: PoolAccounts,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub mint: Pubkey,
}

impl Checklist {
    pub fn all_clear(&self) -> bool {
        !self.is_pump_fun
            && self.lp_burnt
            && self.mint_authority_renounced
            && self.freeze_authority_renounced
            && !self.timeout
            && self.sol_pooled >= 6.9
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct PoolAccounts {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub lp_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub coin_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub pc_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub pool_coin_token_account: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub pool_pc_token_account: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub user_wallet: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub user_token_coin: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub user_token_pc: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub user_lp_token: Pubkey,
}

/// run_checks checks if:
/// 1. the token is a pump fun
/// 2. the pool has enough sol pooled
/// 3. the pool has enough burn pct
/// 4. the token is safe (mint authority + freeze authority)
///     if everything is good, it swaps the token it has the possibility of
///     checking top holders, but this is not relevant the top holders ratio
///     right after creation does not matter as much, as long as it is not
///     a pump fun
pub async fn run_checks(
    signature: String,
) -> Result<(bool, Checklist), Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new_with_commitment(
        env("RPC_URL"),
        CommitmentConfig::processed(),
    );
    let tx = rpc_client
        .get_transaction_with_config(
            &Signature::from_str(&signature)?,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(1),
            },
        )
        .await?;
    let accounts = parse_accounts(&tx)?;
    info!(
        "{}: {}",
        signature,
        serde_json::to_string_pretty(&accounts).unwrap()
    );
    let (ok, checklist) =
        _run_checks(&rpc_client, accounts, tx.slot, true).await?;
    Ok((ok, checklist))
}

pub async fn _run_checks(
    rpc_client: &RpcClient,
    accounts: PoolAccounts,
    slot: u64,
    ignore_non_pump_funs: bool,
) -> Result<(bool, Checklist), Box<dyn std::error::Error>> {
    let (sol_vault, mint) =
        if accounts.coin_mint.eq(&constants::SOLANA_PROGRAM_ID) {
            (accounts.pool_coin_token_account, accounts.pc_mint)
        } else {
            (accounts.pool_pc_token_account, accounts.coin_mint)
        };

    let mut checklist = Checklist {
        slot,
        accounts,
        mint,
        ..Default::default()
    };

    // could be insta-sniping the pump fun launches, generally I am pretty fast
    // (~10 slots) so sniping pumpfuns since they pass all checks is ok
    let is_pump_fun = check_if_pump_fun(&mint).await?;
    checklist.is_pump_fun = is_pump_fun;
    if is_pump_fun {
        return Ok((true, checklist));
    }
    if ignore_non_pump_funs {
        // ignoring any other tokens, way too many scams (noise to profit ratio
        // is too low), even with higher, centralized supply
        // only profit opp is a fair launch of a larger token, but this happens rarely
        // current strat is to flip pumps for 30-50% profit
        return Ok((false, checklist));
    }

    let pubsub_client = PubsubClient::new(&env("WS_URL")).await?;

    let (mut lp_stream, lp_unsub) = pubsub_client
        .account_subscribe(
            &accounts.user_lp_token,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await?;

    let (mut sol_vault_stream, sol_vault_unsub) = pubsub_client
        .account_subscribe(
            &sol_vault,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                ..Default::default()
            }),
        )
        .await?;

    // stream to check total supply, mint authority, freeze authority generally,
    // will run a check if LP burnt, but mint renounce happens sometimes after a
    // delay (user decision)
    let (mut mint_stream, mint_unsub) = pubsub_client
        .account_subscribe(
            &mint,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await?;

    let accounts = &rpc_client
        .get_multiple_accounts(&[accounts.user_lp_token, mint, sol_vault])
        .await?[..];
    if accounts.iter().all(|x| x.is_some()) {
        let account = match accounts[0].clone() {
            Some(account) => account,
            None => {
                return Err("Could not get account user lp account".into());
            }
        };
        let lp_account =
            spl_token::state::Account::unpack(&account.data).unwrap();
        if lp_account.amount == 0 {
            checklist.lp_burnt = true;
        }

        // generally, if checks pass might skip subbing to the mint stream, same with lp stream
        let account = match accounts[1].clone() {
            Some(account) => account,
            None => {
                return Err("Could not get account mint".into());
            }
        };
        let mint_account = Mint::unpack(&account.data).unwrap();
        if mint_account.mint_authority.is_none() {
            checklist.mint_authority_renounced = true;
        }
        if mint_account.freeze_authority.is_none() {
            checklist.freeze_authority_renounced = true;
        }
        if checklist.all_clear() {
            return Ok((true, checklist));
        }

        let account = match accounts[2].clone() {
            Some(account) => account,
            None => {
                return Err("Could not get account sol vault".into());
            }
        };
        let sol_pooled = account.lamports as f64 / 10u64.pow(9) as f64;
        checklist.sol_pooled = sol_pooled;
        // this is the only check that can terminate prematurely
        if sol_pooled < 6.9 {
            return Ok((false, checklist));
        }
    }

    let ok = loop {
        tokio::select! {
            lp_log = lp_stream.next(), if !checklist.lp_burnt => {
                let lp_log = lp_log.unwrap();
                debug!("{} {} lp log received", lp_log.context.slot, &mint);
                if let UiAccountData::Binary(data, UiAccountEncoding::Base64) = lp_log.value.data {
                    let log_data = base64::prelude::BASE64_STANDARD.decode(data).unwrap();
                    if log_data.is_empty() {
                        warn!("empty log data");
                        continue;
                    }
                    let lp_account = spl_token::state::Account::unpack(&log_data).unwrap();
                    if lp_account.amount == 0 {
                        checklist.lp_burnt = true;
                    };
                }
            }
            vault_log = sol_vault_stream.next() => {
                // the amount of sol is there as lamports straight in the log
                let vault_log = vault_log.unwrap();
                debug!("{} {} vault log received", vault_log.context.slot, &mint);
                let sol_pooled = vault_log.value.lamports as f64 / 10u64.pow(9) as f64;
                checklist.sol_pooled = sol_pooled;
                if sol_pooled < 6.9 {
                    break false;
                }
                // this might run for a long time, if no rugpull happens but the
                // mint authority is not renounced, worth adding a timeout
                if checklist.all_clear() {
                    break true;
                }
            }
            mint_log = mint_stream.next(), if !checklist.freeze_authority_renounced || !checklist.mint_authority_renounced => {
                let mint_log = mint_log.unwrap();
                debug!("{} {} mint log received", mint_log.context.slot, &mint);
                if let UiAccountData::Binary(data, UiAccountEncoding::Base64) = mint_log.value.data {
                    let log_data = base64::prelude::BASE64_STANDARD.decode(data).unwrap();
                    let mint_data = Mint::unpack(&log_data).unwrap();
                    if mint_data.mint_authority.is_none() {
                        checklist.mint_authority_renounced = true;
                    }
                    if mint_data.freeze_authority.is_none() {
                        checklist.freeze_authority_renounced = true;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(900)) => {
                info!("timeout");
                checklist.timeout = true;
                break false;
            }
        }
    };

    mint_unsub().await;
    lp_unsub().await;
    sol_vault_unsub().await;

    Ok((ok, checklist))
}

pub fn parse_accounts(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<PoolAccounts, Box<dyn std::error::Error>> {
    if let EncodedTransaction::Json(ui_tx) = &tx.transaction.transaction {
        if let UiMessage::Parsed(UiParsedMessage {
            account_keys: _,
            instructions,
            recent_blockhash: _,
            address_table_lookups: _,
        }) = &ui_tx.message
        {
            for ix in instructions.iter() {
                if let UiInstruction::Parsed(
                    UiParsedInstruction::PartiallyDecoded(
                        UiPartiallyDecodedInstruction {
                            accounts,
                            program_id,
                            data: _,
                            stack_height: _,
                        },
                    ),
                ) = ix
                {
                    if accounts.len() == 21
                        && program_id
                            == &constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                                .to_string()
                    {
                        let amm_pool = Pubkey::from_str(&accounts[4]).unwrap();
                        let lp_mint = Pubkey::from_str(&accounts[7]).unwrap();
                        let coin_mint =
                            Pubkey::from_str(&accounts[8]).unwrap();
                        let pc_mint = Pubkey::from_str(&accounts[9]).unwrap();
                        let pool_coin_token_account =
                            Pubkey::from_str(&accounts[10]).unwrap();
                        let pool_pc_token_account =
                            Pubkey::from_str(&accounts[11]).unwrap();
                        let user_wallet =
                            Pubkey::from_str(&accounts[17]).unwrap();
                        let user_token_coin =
                            Pubkey::from_str(&accounts[18]).unwrap();
                        let user_token_pc =
                            Pubkey::from_str(&accounts[19]).unwrap();
                        let user_lp_token =
                            Pubkey::from_str(&accounts[20]).unwrap();

                        return Ok(PoolAccounts {
                            amm_pool,
                            lp_mint,
                            coin_mint,
                            pc_mint,
                            pool_coin_token_account,
                            pool_pc_token_account,
                            user_wallet,
                            user_token_coin,
                            user_token_pc,
                            user_lp_token,
                        });
                    }
                }
            }
        }
    }
    Err("Could not parse accounts".into())
}

#[cfg(test)]
mod tests {
    use solana_sdk::program_pack::Pack;

    #[tokio::test]
    async fn test_run_checks() {
        let signature = "2cbovtqtKSGgEcrTkg2AV4h5aC3mRt3QfrWwnn4dccAehjMfptMCLxRpdWsRJ2XWafCuqcR6AWQC1ieq4E13xrap".to_string();
        super::run_checks(signature).await.unwrap();
    }

    #[test]
    fn test_unpack_mint() {
        let data = "1111Dk7tnoddMvATwtoKYbhf9c51kPxy4Siv5Ubb93zssnpGt5j2ELBnz1TT5a7jGAeKE9zEsoFAY5kByXAhfi8EYHCg3ChYCmZ6rnyNYPxQrK".to_string();
        let _ = super::Mint::unpack(
            bs58::decode(data).into_vec().unwrap().as_slice(),
        )
        .unwrap();
    }
}
