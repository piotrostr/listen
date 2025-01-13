use crate::{
    raydium::{parse_holding, Holding},
    types,
    util::env,
};
use std::str::FromStr;

use log::{debug, info, warn};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction,
    rpc_config::RpcTransactionConfig, rpc_request::TokenAccountsFilter,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};
use spl_token_2022::{
    extension::StateWithExtensionsOwned,
    state::{Account, Mint},
};
use timed::timed;

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new_with_commitment(
        url.to_string(),
        CommitmentConfig::processed(),
    );

    Ok(rpc_client)
}

// Provider provides the data, contains both RPC client that can
// communicate over the REST interface and utilities like getting
// the pricing data from Jupiter
pub struct Provider {}

impl Provider {
    #[timed(duration(printer = "info!"))]
    pub async fn get_holdings(
        rpc_client: &RpcClient,
        owner: &Pubkey,
    ) -> Result<Vec<Holding>, Box<dyn std::error::Error>> {
        let atas = rpc_client
            .get_token_accounts_by_owner(
                owner,
                TokenAccountsFilter::ProgramId(spl_token::id()),
            )
            .await?;
        info!("found {} token accounts", atas.len());
        let holdings = atas
            .iter()
            .map(|ata| parse_holding(ata.clone()).expect("parse holding"))
            .filter(|holding| holding.amount > 0)
            .collect::<Vec<Holding>>();

        Ok(holdings)
    }

    #[timed(duration(printer = "info!"))]
    pub async fn get_balance(
        rpc_client: &RpcClient,
        pubkey: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let balance = rpc_client.get_balance(pubkey).await?;
        Ok(balance)
    }

    #[timed(duration(printer = "info!"))]
    pub async fn get_spl_balance(
        rpc_client: &RpcClient,
        pubkey: &Pubkey,
        mint: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let token_accounts = rpc_client
            .get_token_accounts_by_owner(
                pubkey,
                TokenAccountsFilter::Mint(*mint),
            )
            .await?;
        match token_accounts.first() {
            Some(token_account) => {
                let acount_info = rpc_client
                    .get_account(&Pubkey::from_str(
                        token_account.pubkey.as_str(),
                    )?)
                    .await?;
                let token_account_info = Account::unpack(&acount_info.data)?;
                debug!("Token account info: {:?}", token_account_info);
                Ok(token_account_info.amount)
            }
            None => Err("No token account found".into()),
        }
    }

    #[timed(duration(printer = "info!"))]
    pub async fn get_tx(
        rpc_client: &RpcClient,
        signature: &str,
    ) -> Result<
        EncodedConfirmedTransactionWithStatusMeta,
        Box<dyn std::error::Error>,
    > {
        let sig = Signature::from_str(signature)?;
        let mut backoff = 100;
        let retries = 5;
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
                    debug!("Error getting tx: {:?}", e);
                    std::thread::sleep(std::time::Duration::from_millis(
                        backoff,
                    ));
                    backoff *= 2;
                }
            }
        }
        Err(format!("could not fetch {}", signature).into())
    }

    #[timed(duration(printer = "info!"))]
    pub async fn get_pricing(
        mint: &str,
    ) -> Result<types::PriceResponse, Box<dyn std::error::Error>> {
        let url = format!("https://api.jup.ag/price/v2?ids={}", mint);
        let client = reqwest::Client::new();
        let res = client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await?;
        let data = res.json::<types::PriceResponse>().await?;
        Ok(data)
    }

    #[timed(duration(printer = "info!"))]
    pub async fn send_tx(
        rpc_client: &RpcClient,
        tx: &impl SerializableTransaction,
        _skip_preflight: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        match rpc_client
            .send_transaction(
                tx,
                // CommitmentConfig::processed(),
                // RpcSendTransactionConfig {
                //     skip_preflight,
                //     ..RpcSendTransactionConfig::default()
                // },
            )
            .await
        {
            Ok(signature) => {
                info!("Sent in: {:?}", start.elapsed());
                Ok(signature.to_string())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// sanity_check is for mint_authority and freeze_authority, for non
    /// pump.fun tokens is crucial, mint authority enables minting any amount of
    /// the token and freeze authority can renounce the ability to trade the
    /// token for a given address
    #[timed(duration(printer = "info!"))]
    pub async fn sanity_check(
        rpc_client: &RpcClient,
        mint: &Pubkey,
    ) -> Result<(bool, String), Box<dyn std::error::Error>> {
        let account = rpc_client.get_account(mint).await?;
        // recommended approach
        // get the token account mint based on the account too to confirm
        // skipping this check for the time being
        let state = StateWithExtensionsOwned::<Mint>::unpack(account.data)
            .expect("unpack mint");
        if state.base.mint_authority.is_some() {
            return Ok((
                false,
                "mint authority has not been renounced".to_string(),
            ));
        }
        if state.base.freeze_authority.is_some() {
            return Ok((
                false,
                "freeze authority has not been renounced".to_string(),
            ));
        }

        Ok((true, "ok".to_string()))
    }
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

pub async fn get_tx_async(
    signature: &str,
) -> Result<
    EncodedConfirmedTransactionWithStatusMeta,
    Box<dyn std::error::Error>,
> {
    let rpc_client = RpcClient::new(env("RPC_URL"));
    let sig = Signature::from_str(signature)?;
    let mut backoff = 100;
    let retries = 5;
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
                debug!("Error getting tx: {:?}", e);
                std::thread::sleep(std::time::Duration::from_millis(backoff));
                backoff *= 2;
            }
        }
    }
    Err(format!("could not fetch {}", signature).into())
}
