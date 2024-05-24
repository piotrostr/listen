use std::str::FromStr;

use dotenv_codegen::dotenv;
use serde::{Deserialize, Serialize};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction, UiParsedMessage,
    UiPartiallyDecodedInstruction, UiTransactionEncoding,
};

use crate::constants;

#[derive(Serialize, Deserialize)]
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

// Helper functions for serialization and deserialization
fn pubkey_to_string<S>(
    pubkey: &Pubkey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&pubkey.to_string())
}

fn string_to_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

pub async fn run_checks(
    signature: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new_with_commitment(
        dotenv!("RPC_URL").to_string(),
        CommitmentConfig::confirmed(),
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
    println!("{}", serde_json::to_string_pretty(&accounts).unwrap());

    Ok(())
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
                            == constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                    {
                        let amm_pool = Pubkey::from_str(&accounts[4]).unwrap();
                        let lp_mint = Pubkey::from_str(&accounts[7]).unwrap();
                        let coin_mint = Pubkey::from_str(&accounts[8]).unwrap();
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
    #[tokio::test]
    async fn test_run_checks() {
        let signature = "2cbovtqtKSGgEcrTkg2AV4h5aC3mRt3QfrWwnn4dccAehjMfptMCLxRpdWsRJ2XWafCuqcR6AWQC1ieq4E13xrap".to_string();
        super::run_checks(signature).await.unwrap();
    }
}
