use crate::constants;

use serde::Serialize;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::{LogsSubscription, PubsubClient},
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
        RpcProgramAccountsConfig, RpcTransactionLogsConfig,
        RpcTransactionLogsFilter,
    },
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{str::FromStr, time::Duration};

pub struct Listener {
    ws_url: String,
}

#[derive(Debug, Serialize, Default)]
pub struct Swap {
    pub signature: String,

    pub quote_amount: f64,
    pub quote_mint: String,

    pub base_amount: f64,
    pub base_mint: String,

    pub sol_amount_ui: f64,
}

trait BlockAndProgramSubscribable {
    fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn program_subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Listener {
    pub fn new(ws_url: String) -> Listener {
        Listener { ws_url }
    }

    pub fn logs_subscribe(
        &self,
    ) -> Result<LogsSubscription, Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };
        let filter =
            RpcTransactionLogsFilter::Mentions(
                vec![raydium_pubkey.to_string()],
            );
        let (subs, receiver) = PubsubClient::logs_subscribe(
            &self.ws_url.as_str(),
            filter,
            config,
        )?;

        println!("Connecting to logs for {:?}", raydium_pubkey);
        Ok((subs, receiver))
    }
}

impl BlockAndProgramSubscribable for Listener {
    fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;

        let filter = RpcBlockSubscribeFilter::MentionsAccountOrProgram(
            raydium_pubkey.to_string(),
        );
        let config = RpcBlockSubscribeConfig::default();

        let (mut subs, receiver) = PubsubClient::block_subscribe(
            self.ws_url.as_str(),
            filter,
            Some(config),
        )?;

        println!("Filtering for mentions of {:?}", raydium_pubkey);

        while let Ok(block) = receiver.recv_timeout(Duration::from_secs(1)) {
            println!("Received block: {:?}", block);
        }

        subs.shutdown().unwrap();

        Ok(())
    }

    fn program_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
        let mut config = RpcProgramAccountsConfig::default();
        config.account_config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::JsonParsed),
            data_slice: None,
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: None,
        };
        let (mut subs, receiver) = PubsubClient::program_subscribe(
            &self.ws_url.as_str(),
            &raydium_pubkey,
            Some(config),
        )?;

        println!("Connecting to program {:?}", raydium_pubkey);

        let mut i = 0;
        while let Ok(account) = receiver.recv_timeout(Duration::from_secs(1)) {
            i += 1;
            println!("Received account: {:?}", account);
            if i == 1 {
                break;
            }
        }
        subs.shutdown().unwrap();

        Ok(())
    }
}
