use crate::constants;

use log::info;
use serde::Serialize;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::{LogsSubscription, PubsubClient},
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig,
        RpcBlockSubscribeFilter, RpcProgramAccountsConfig,
        RpcTransactionLogsConfig, RpcTransactionLogsFilter,
    },
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::time::Duration;

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

pub trait BlockAndProgramSubscribable {
    fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn program_subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn slot_subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Listener {
    pub fn new(ws_url: String) -> Listener {
        Listener { ws_url }
    }

    pub fn account_subscribe(
        &self,
        pubkey: &Pubkey,
    ) -> Result<LogsSubscription, Box<dyn std::error::Error>> {
        let (subs, receiver) = PubsubClient::logs_subscribe(
            self.ws_url.as_str(),
            RpcTransactionLogsFilter::Mentions(vec![pubkey.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::processed()),
            },
        )?;
        Ok((subs, receiver))
    }

    pub fn pool_subscribe(
        &self,
        amm_pool: &Pubkey,
    ) -> Result<LogsSubscription, Box<dyn std::error::Error>> {
        let (subs, receiver) = PubsubClient::logs_subscribe(
            self.ws_url.as_str(),
            RpcTransactionLogsFilter::Mentions(vec![amm_pool.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::processed()),
            },
        )?;
        Ok((subs, receiver))
    }

    pub fn logs_subscribe(
        &self,
    ) -> Result<LogsSubscription, Box<dyn std::error::Error>> {
        let raydium_pubkey = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };
        let filter = RpcTransactionLogsFilter::Mentions(vec![
            raydium_pubkey.to_string()
        ]);
        let (subs, receiver) = PubsubClient::logs_subscribe(
            self.ws_url.as_str(),
            filter,
            config,
        )?;

        info!("listening to logs for {:?}", raydium_pubkey);
        Ok((subs, receiver))
    }
}

impl BlockAndProgramSubscribable for Listener {
    fn slot_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (mut subs, receiver) =
            PubsubClient::slot_subscribe(self.ws_url.as_str())?;
        info!("listening to slots over {}", self.ws_url);

        if let Ok(slot) = receiver.recv() {
            let mut ts = tokio::time::Instant::now();
            info!("starting slot: {:?}", slot);
            while let Ok(slot) = receiver.recv() {
                info!(
                    "slot: {:?} in {}ms",
                    slot.slot,
                    ts.elapsed().as_millis()
                );
                ts = tokio::time::Instant::now();
            }
        }

        subs.shutdown().unwrap();

        Ok(())
    }
    fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;

        let filter = RpcBlockSubscribeFilter::MentionsAccountOrProgram(
            raydium_pubkey.to_string(),
        );
        let config = RpcBlockSubscribeConfig::default();

        let (mut subs, receiver) = PubsubClient::block_subscribe(
            self.ws_url.as_str(),
            filter,
            Some(config),
        )?;

        info!("Filtering for mentions of {:?}", raydium_pubkey);

        while let Ok(block) = receiver.recv_timeout(Duration::from_secs(1)) {
            info!("Received block: {:?}", block);
        }

        subs.shutdown().unwrap();

        Ok(())
    }

    fn program_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
        let config = RpcProgramAccountsConfig {
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::JsonParsed),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
            ..RpcProgramAccountsConfig::default()
        };
        let (mut subs, receiver) = PubsubClient::program_subscribe(
            self.ws_url.as_str(),
            &raydium_pubkey,
            Some(config),
        )?;

        info!("listening on program {:?}", raydium_pubkey);

        let mut i = 0;
        while let Ok(account) = receiver.recv_timeout(Duration::from_secs(1)) {
            i += 1;
            info!("Received account: {:?}", account);
            if i == 1 {
                break;
            }
        }
        subs.shutdown().unwrap();

        Ok(())
    }
}
