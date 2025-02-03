use anyhow::{anyhow, Result};
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use serde::Deserialize;
use solana_account_decoder::parse_account_data::ParsedAccount;
use solana_account_decoder::UiAccountData;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use std::future::Future;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

use crate::common::wrap_unsafe;
use crate::signer::solana::LocalSolanaSigner;
use crate::signer::{SignerContext, TransactionSigner};

pub fn env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{} env var not set", var))
}

/// Helper function for pubkey serialize
pub fn pubkey_to_string<S>(
    pubkey: &Pubkey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&pubkey.to_string())
}

/// Helper function for pubkey deserialize
pub fn string_to_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

pub fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub fn make_compute_budget_ixs(
    price: u64,
    max_units: u32,
) -> Vec<Instruction> {
    vec![
        ComputeBudgetInstruction::set_compute_unit_price(price),
        ComputeBudgetInstruction::set_compute_unit_limit(max_units),
    ]
}

#[derive(Debug, Default, Clone)]
pub struct Holding {
    pub mint: Pubkey,
    pub ata: Pubkey,
    pub amount: u64,
}

pub fn parse_holding(ata: RpcKeyedAccount) -> Result<Holding> {
    if let UiAccountData::Json(ParsedAccount {
        program: _,
        parsed,
        space: _,
    }) = ata.account.data
    {
        let amount = parsed["info"]["tokenAmount"]["amount"]
            .as_str()
            .expect("amount")
            .parse::<u64>()?;
        let mint =
            Pubkey::from_str(parsed["info"]["mint"].as_str().expect("mint"))?;
        let ata = Pubkey::from_str(&ata.pubkey)?;
        Ok(Holding { mint, ata, amount })
    } else {
        Err(anyhow!("failed to parse holding"))
    }
}

pub fn init_logger() -> Result<()> {
    let logs_level = match std::env::var("RUST_LOG") {
        Ok(level) => {
            LevelFilter::from_str(&level).unwrap_or(LevelFilter::Info)
        }
        Err(_) => LevelFilter::Info,
    };

    // in logs, use unix timestamp in ms
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().timestamp_millis(),
                record.level(),
                record.args()
            )
        })
        .filter(None, logs_level)
        .try_init()?;

    Ok(())
}

pub fn apply_fee(amount: u64) -> u64 {
    amount * 101 / 100
}

pub fn load_keypair_for_tests() -> Keypair {
    Keypair::from_base58_string(&env("SOLANA_PRIVATE_KEY"))
}

pub fn make_test_signer() -> Arc<dyn TransactionSigner> {
    Arc::new(LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY")))
}

pub fn make_rpc_client() -> RpcClient {
    let rpc_url = env("SOLANA_RPC_URL");
    RpcClient::new(rpc_url)
}

pub async fn verify_transaction(
    signature: &str,
    rpc_client: &RpcClient,
) -> bool {
    // Wait for transaction confirmation
    let confirmation = rpc_client
        .confirm_transaction_with_commitment(
            &signature.parse().unwrap(),
            CommitmentConfig::confirmed(),
        )
        .await;

    tracing::debug!(?signature, ?confirmation);

    match confirmation {
        Ok(resp) => resp.value,
        Err(_) => false,
    }
}

pub fn parse_pubkey(s: &str) -> Result<Pubkey> {
    match Pubkey::from_str(s) {
        Ok(pubkey) => Ok(pubkey),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn execute_solana_transaction<F, Fut>(
    tx_creator: F,
) -> Result<String>
where
    F: FnOnce(Pubkey) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Transaction>> + Send + 'static,
{
    let signer = SignerContext::current().await;
    let owner = Pubkey::from_str(&signer.pubkey())?;

    let mut tx = wrap_unsafe(move || async move { tx_creator(owner).await })
        .await
        .map_err(|e| anyhow!("{:#?}", e))?;

    wrap_unsafe(move || async move {
        signer.sign_and_send_solana_transaction(&mut tx).await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}
