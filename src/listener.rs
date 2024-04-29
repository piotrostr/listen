use crate::constants;

use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
        RpcProgramAccountsConfig, RpcTransactionLogsConfig,
        RpcTransactionLogsFilter,
    },
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction,
};
use std::{str::FromStr, time::Duration};
pub struct Listener {
    ws_url: String,
}

impl Listener {
    pub fn new(ws_url: String) -> Listener {
        Listener { ws_url }
    }

    pub fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;

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

    pub fn program_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;
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

    pub fn logs_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raydium_pubkey =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };
        let filter =
            RpcTransactionLogsFilter::Mentions(
                vec![raydium_pubkey.to_string()],
            );
        let (mut subs, receiver) = PubsubClient::logs_subscribe(
            &self.ws_url.as_str(),
            filter,
            config,
        )?;

        println!("Connecting to logs for {:?}", raydium_pubkey);

        if let Ok(log) = receiver.recv_timeout(Duration::from_secs(1)) {
            println!("{}", serde_json::to_string_pretty(&log)?);
        }

        subs.shutdown().unwrap();

        Ok(())
    }

    pub fn parse_mint(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let instructions = self.parse_instructions(tx);
        for instruction in instructions {
            match instruction {
                UiInstruction::Parsed(ix) => match ix {
                    UiParsedInstruction::Parsed(ix) => {
                        if ix.program == "spl-associated-token-account" {
                            // TODO this might panic, might be handled more gracefully
                            let mint = ix.parsed["info"]["mint"]
                                .as_str()
                                .unwrap()
                                .to_string();
                            return Ok(mint);
                        }
                    }
                    UiParsedInstruction::PartiallyDecoded(_) => (),
                },
                _ => (),
            }
        }
        return Err("Mint not found in tx".into());
    }

    pub fn parse_instructions(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Vec<UiInstruction> {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Parsed(msg) => msg.instructions.clone(),
                UiMessage::Raw(_) => core::panic!("Unsupported raw message"),
            },
            _ => core::panic!("Unsupported transaction encoding"),
        }
    }
}
