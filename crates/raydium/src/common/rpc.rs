use anyhow::Result;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
    rpc_request::RpcRequest,
    rpc_response::{RpcResult, RpcSimulateTransactionResult},
};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey,
    signature::Signature, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

pub async fn send_txn(
    client: &RpcClient,
    txn: &Transaction,
    skip_preflight: bool,
) -> Result<Signature> {
    Ok(client
        .send_and_confirm_transaction_with_spinner_and_config(
            txn,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight,
                ..RpcSendTransactionConfig::default()
            },
        )
        .await?)
}

pub async fn simulate_transaction(
    client: &RpcClient,
    transaction: &Transaction,
    sig_verify: bool,
    cfg: CommitmentConfig,
) -> RpcResult<RpcSimulateTransactionResult> {
    let serialized_encoded =
        base64::encode(bincode::serialize(transaction).unwrap());
    client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": sig_verify, "commitment": cfg.commitment, "encoding": Some(UiTransactionEncoding::Base64)
        }]),
    ).await
}

pub async fn send_without_confirm_txn(
    client: &RpcClient,
    txn: &Transaction,
) -> Result<Signature> {
    Ok(client
        .send_transaction_with_config(
            txn,
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )
        .await?)
}

pub async fn get_account<T>(
    client: &RpcClient,
    addr: &Pubkey,
) -> Result<Option<T>>
where
    T: Clone,
{
    if let Some(account) = client
        .get_account_with_commitment(addr, CommitmentConfig::processed())
        .await?
        .value
    {
        let account_data = account.data.as_slice();
        let ret = unsafe { &*(&account_data[0] as *const u8 as *const T) };
        Ok(Some(ret.clone()))
    } else {
        Ok(None)
    }
}

pub fn deserialize_account<T: Copy>(
    account: &Account,
    is_anchor_account: bool,
) -> Result<T> {
    let mut account_data = account.data.as_slice();
    if is_anchor_account {
        account_data = &account_data[8..std::mem::size_of::<T>() + 8];
    }
    Ok(unsafe { *(&account_data[0] as *const u8 as *const T) })
}

pub async fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys).await?)
}