pub mod constants;

use actix_web::{get, HttpResponse, Responder};
use log::{debug, info, warn};
use serde::Deserialize;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey, signature::Signature
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};
use std::str::FromStr;

pub fn env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{} env var not set", var))
}

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1000000000.0
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1000000000.0) as u64
}

#[get("/healthz")]
pub async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("im ok")
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

pub fn max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
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
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(price),
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(max_units),
    ]
}

pub async fn get_tx_async_with_client(
    rpc_client: &RpcClient,
    signature: &str,
    retries: u32,
) -> Result<
    EncodedConfirmedTransactionWithStatusMeta,
    Box<dyn std::error::Error>,
> {
    let sig = Signature::from_str(signature)?;
    let mut backoff = 100;
    for _ in 0..retries {
        match rpc_client
            .get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(1),
                },
            )
            .await
        {
            Ok(tx) => return Ok(tx),
            Err(e) => {
                warn!("Error getting tx: {:?}", e);
                std::thread::sleep(std::time::Duration::from_millis(backoff));
                backoff *= 2;
            }
        }
    }
    Err(format!("could not fetch {}", signature).into())
}
