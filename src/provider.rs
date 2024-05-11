use crate::{constants, types};
use std::str::FromStr;

use log::{debug, info};
use solana_client::{
    rpc_client::{RpcClient, SerializableTransaction},
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig,
        RpcSendTransactionConfig, RpcTransactionConfig,
    },
    rpc_filter::RpcFilterType,
    rpc_request::TokenAccountsFilter,
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

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client =
        RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let identity = rpc_client.get_identity()?;

    info!("Connecting to blocks through {:?}", url);
    info!("Latest blockhash: {:?}", latest_blockhash);
    info!("Identity: {:?}", identity);

    Ok(rpc_client)
}

// Provider provides the data, contains both RPC client that can
// communicate over the REST interface and utilities like getting
// the pricing data from Jupiter
pub struct Provider {
    pub rpc_client: RpcClient,
}

impl Provider {
    pub fn new(rpc_url: String) -> Provider {
        Provider {
            rpc_client: get_client(rpc_url.as_str()).unwrap(),
        }
    }

    pub fn get_balance(
        &self,
        pubkey: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let balance = self.rpc_client.get_balance(pubkey)?;
        Ok(balance)
    }

    pub fn get_spl_balance(
        &self,
        pubkey: &Pubkey,
        mint: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let token_accounts = self.rpc_client.get_token_accounts_by_owner(
            pubkey,
            TokenAccountsFilter::Mint(*mint),
        )?;
        match token_accounts.first() {
            Some(token_account) => {
                let acount_info = self.rpc_client.get_account(
                    &Pubkey::from_str(token_account.pubkey.as_str())?,
                )?;
                let token_account_info = Account::unpack(&acount_info.data)?;
                debug!("Token account info: {:?}", token_account_info);
                Ok(token_account_info.amount)
            }
            None => Err("No token account found".into()),
        }
    }

    pub fn get_tx(
        &self,
        signature: &str,
    ) -> Result<
        EncodedConfirmedTransactionWithStatusMeta,
        Box<dyn std::error::Error>,
    > {
        let sig = Signature::from_str(signature)?;
        let mut backoff = 100;
        let retries = 5;
        for _ in 0..retries {
            match self.rpc_client.get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(1),
                },
            ) {
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

    pub async fn get_pricing(
        &self,
        mint: &str,
    ) -> Result<types::PriceResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://price.jup.ag/v4/price?ids={}&vsToken={}",
            mint,
            constants::SOLANA_PROGRAM_ID,
        );
        debug!("Getting pricing from: {:?}", url);
        let client = reqwest::Client::new();
        let res = client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await?;
        let data = res.json::<types::PriceResponse>().await?;
        Ok(data)
    }

    pub fn send_tx(
        &self,
        tx: &impl SerializableTransaction,
        skip_preflight: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        match self.rpc_client.send_transaction_with_config(
            tx,
            RpcSendTransactionConfig {
                skip_preflight,
                ..RpcSendTransactionConfig::default()
            },
        ) {
            Ok(signature) => {
                info!("Sent in: {:?}", start.elapsed());
                Ok(signature.to_string())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn sanity_check(
        &self,
        mint: &Pubkey,
    ) -> Result<(bool, String), Box<dyn std::error::Error>> {
        let account = self.rpc_client.get_account(mint)?;
        // recommended approach
        // get the token account mint based on the account too to confirm
        // skipping this check for fast testing
        let state = StateWithExtensionsOwned::<Mint>::unpack(account.data)?;
        // could also retry with the spl_token, using spl_token_2022 above, but
        // I assume backwards compatibility
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
