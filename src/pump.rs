use std::error::Error;
use std::str::FromStr;
use base64::Engine;
use futures_util::StreamExt;
use log::info;

use serde::{Deserialize, Serialize};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::nonce_utils::nonblocking::get_account_with_commitment;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage, UiParsedMessage};

use crate::constants::SOLANA_PROGRAM_ID;
use crate::get_tx_async;
use crate::raydium::make_compute_budget_ixs;
use crate::util::{env, pubkey_to_string, string_to_pubkey};

pub const BLOXROUTE_ADDRESS: &str = "HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY";
pub const PUMP_GLOBAL_ADDRESS: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FEE_ADDRESS: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";
pub const PUMP_FUN_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
pub const PUMP_FUN_MINT_AUTHORITY: &str = "TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM";
pub const EVENT_AUTHORITY: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";

pub async fn buy_pump_token(pump_accounts: PumpAccounts, lamports: u64) -> Result<(), Box<dyn Error>> {
    info!("Buying pump token {}", pump_accounts.mint.to_string());
    let wallet = Keypair::read_from_file("./fuck.json").expect("read wallet");
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let owner = wallet.pubkey();

    let mut ixs = vec![];
    ixs.append(&mut make_compute_budget_ixs(262500, 100000));
    // bloxroute might be required, it is used by pump but not sure if crucial they are probably an
    // enterprise user and that is why they are using it jito is probably fine, but jito rust sucks
    // coz of the stupid searcher_client lolz
    // 
    // 0.003 sol
    // let tip = 3000000;
    // ixs.push(solana_sdk::system_instruction::transfer(
    //     &owner,
    //     &Pubkey::from_str(BLOXROUTE_ADDRESS)?,
    //     tip,
    // ));
    let _ /* ata */ = &spl_associated_token_account::get_associated_token_address(&owner, &pump_accounts.mint);
    let mut ata_ixs = raydium_library::common::create_ata_token_or_not(&owner, &pump_accounts.mint, &owner);

    ixs.append(&mut ata_ixs);
    ixs.push(make_pump_swap_ix(owner, pump_accounts, 0, lamports)?);

    let recent_blockhash = rpc_client.get_latest_blockhash().await?;

    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &ixs,
        Some(&owner),
        &[&wallet],
        recent_blockhash
    );

    println!("signed: {}", transaction.is_signed());

    // send the tx
    let signature = rpc_client.send_and_confirm_transaction_with_spinner(&transaction).await?;

    info!("Transaction signature: {}", signature.to_string());

    Ok(())
}

/// Interact With Pump.Fun 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
/// Input Accounts
/// #1 - Global: 4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf
/// #2 - Fee Recipient: Pump.fun Fee Account [Writable]
/// #3 - Mint 
/// #4 - Bonding Curve [Writable]
/// #5 - Associated Bonding Curve [Writable]
/// #6 - Associated User [Writable]
/// #7 - User - owner, sender [Writable, Signer, Fee Payer]
/// #8 - System Program (11111111111111111111111111111111)
/// #9 - Token Program (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
/// #10 - Rent (SysvarRent111111111111111111111111111111111)
/// #11 - Event Authority: Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1
/// #12 - Program: Pump.fun Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
pub fn make_pump_swap_ix(owner: Pubkey, pump_accounts: PumpAccounts, token_amount: u64, lamports: u64) -> Result<Instruction, Box<dyn Error>> {
    let accounts = vec![
        AccountMeta::new_readonly(Pubkey::from_str(PUMP_GLOBAL_ADDRESS)?, false),
        AccountMeta::new(Pubkey::from_str(PUMP_FEE_ADDRESS)?, false), // writable
        AccountMeta::new_readonly(pump_accounts.mint, false),
        AccountMeta::new(pump_accounts.bonding_curve, false), // writable
        AccountMeta::new(pump_accounts.associated_bonding_curve, false), // writable
        AccountMeta::new(pump_accounts.associated_user, false), // writable
        AccountMeta::new(owner, true), // signer
        AccountMeta::new_readonly(Pubkey::from_str(SOLANA_PROGRAM_ID)?, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(Pubkey::from_str(EVENT_AUTHORITY)?, false),
        AccountMeta::new_readonly(Pubkey::from_str(PUMP_FUN_PROGRAM)?, false),
    ];
    let mut data = Vec::new();
    data.extend_from_slice(&[0u8; 8]); // Replace with appropriate opcode
    data.extend_from_slice(&token_amount.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());

    Ok(Instruction {
        program_id: Pubkey::from_str(PUMP_FUN_PROGRAM)?,
        accounts,
        data,
    })
}

pub async fn listen_pump() -> Result<(), Box<dyn Error>> {
    let client = PubsubClient::new(&env("WS_URL"))
        .await
        .expect("pubsub client async");
    let (mut notifications, unsub) = client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![PUMP_FUN_MINT_AUTHORITY.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .await
        .expect("subscribe to logs");
    info!("Listening for PumpFun events");
    while let Some(log) = notifications.next().await {
        let sig = log.value.signature;
        let tx = get_tx_async(&sig).await?;
        let accounts = parse_pump_accounts(tx)?;
        println!("{}: {}", sig, serde_json::to_string_pretty(&accounts).unwrap());
        println!("Fetching metadata");
    }
    unsub().await;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PumpAccounts {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub bonding_curve: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub associated_bonding_curve: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub associated_user: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub metadata: Pubkey,
}

/// this kinda works but not using since metadata takes some time to populate
pub async fn fetch_metadata(rpc_client: &RpcClient ,metadata: Pubkey) -> Result<(), Box<dyn Error>> {
    let acc = get_account_with_commitment(rpc_client, &metadata, CommitmentConfig::confirmed()).await?;
    let account_data = base64::prelude::BASE64_STANDARD.decode(acc.data).expect("decode spl b64");
    println!("{:?}", account_data);

    Ok(())
}

pub fn parse_pump_accounts(tx: EncodedConfirmedTransactionWithStatusMeta) -> Result<PumpAccounts, Box<dyn Error>> {
    if let EncodedTransaction::Json(tx) = &tx.transaction.transaction {
        if let UiMessage::Parsed(UiParsedMessage {
            account_keys,
            instructions: _,
            recent_blockhash: _,
            address_table_lookups: _,
        }) = &tx.message {
            if account_keys.len() >= 5 {
                let associated_user = account_keys[0].pubkey.parse()?;
                let mint = account_keys[1].pubkey.parse()?;
                let bonding_curve = account_keys[3].pubkey.parse()?;
                let associated_bonding_curve = account_keys[4].pubkey.parse()?;
                let metadata = account_keys[5].pubkey.parse()?;

                Ok(PumpAccounts {
                    mint,
                    bonding_curve,
                    associated_bonding_curve,
                    associated_user,
                    metadata,
                })
            } else {
                Err("Not enough account keys".into())
            }
        } else {
            Err("Not a parsed transaction".into())
        }
    } else {
        Err("Not a JSON transaction".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pump_accounts() {
        let sample_tx = std::fs::read_to_string("pump_fun_tx.json").expect("read tx");
        let tx: EncodedConfirmedTransactionWithStatusMeta = serde_json::from_str(&sample_tx).expect("parse tx");
        let accounts = parse_pump_accounts(tx).expect("parse accounts");
        println!("{:?}", accounts);
        assert!(accounts.mint.to_string() == "6kPvKNrLqg23mApAvHzMKWohhVdSrA54HvrpYud8pump");
        assert!(accounts.bonding_curve.to_string() == "6TGz5VAFF6UpSmTSk9327utugSWJCyVeVVFXDtZnMtNp");
        assert!(accounts.associated_bonding_curve.to_string() == "4VwNGUif2ubbPjx4YNHmxEH7L4Yt2QFeo8uVTrVC3F68");
        assert!(accounts.associated_user.to_string() == "2wgo94ZaiUNUkFBSKNaKsUgEANgSdex7gRpFKR39DPzw");
    }

    // #[ignore = "comment out when checking, this sends actual buy, havent ran it yet"]
    #[tokio::test]
    async fn test_buy_pump_token() {
        // 0.01 sol
        let lamports = 10000000;
        let pump_accounts = PumpAccounts {
            mint: Pubkey::from_str("6kPvKNrLqg23mApAvHzMKWohhVdSrA54HvrpYud8pump").expect("parse mint"),
            bonding_curve: Pubkey::from_str("6TGz5VAFF6UpSmTSk9327utugSWJCyVeVVFXDtZnMtNp").expect("parse bonding curve"),
            associated_bonding_curve: Pubkey::from_str("4VwNGUif2ubbPjx4YNHmxEH7L4Yt2QFeo8uVTrVC3F68").expect("parse associated bonding curve"),
            associated_user: Pubkey::from_str("2wgo94ZaiUNUkFBSKNaKsUgEANgSdex7gRpFKR39DPzw").expect("parse associated user"),
            metadata: Pubkey::default(), // not required
        };
        buy_pump_token(pump_accounts, lamports).await.expect("buy pump token");
    }
}
