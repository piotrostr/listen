use crate::constants;

use serde::Serialize;
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
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction,
};
use std::{str::FromStr, time::Duration};
pub struct Listener {
    ws_url: String,
}

#[derive(Debug, Serialize)]
pub struct TokenTransfer {
    pub amount: String,
    pub mint: String,
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
        let instructions = self.parse_instructions(tx)?;
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

    pub fn parse_tmp_account(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let instructions = self.parse_instructions(tx)?;
        let mut tmp_account = String::new();
        for instruction in instructions {
            match instruction {
                UiInstruction::Parsed(ix) => match ix {
                    UiParsedInstruction::Parsed(ix) => {
                        if ix.program == "spl-token"
                            && ix.parsed["type"] == "closeAccount"
                        {
                            tmp_account =
                                ix.parsed["info"]["account"].to_string();
                        }
                    }
                    UiParsedInstruction::PartiallyDecoded(_) => {}
                },
                _ => (),
            }
        }

        if tmp_account == "" {
            return Err("Temp account not found".into());
        }

        Ok(tmp_account)
    }

    pub fn parse_token_transfers(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Vec<TokenTransfer>, Box<dyn std::error::Error>> {
        let tmp_account = self.parse_tmp_account(tx)?;
        let quote_mint = self.parse_mint(tx)?;
        let mut res: Vec<TokenTransfer> = vec![];
        if let Some(meta) = &tx.transaction.meta {
            match meta.inner_instructions.clone() {
                OptionSerializer::Some(all_ixs) => {
                    for ixs in all_ixs {
                        // might also be identified based on static index 5 but
                        // that would be even more brittle than this
                        if ixs.instructions.len() == 2 {
                            for ix in ixs.instructions {
                                match ix {
                                    UiInstruction::Parsed(
                                        UiParsedInstruction::Parsed(parsed_ix),
                                    ) => {
                                        if parsed_ix.program == "spl-token"
                                            && parsed_ix.parsed["type"]
                                                == "transfer"
                                        {
                                            let mint = if parsed_ix.parsed
                                                ["info"]["destination"]
                                                == tmp_account
                                            {
                                                quote_mint.clone()
                                            } else {
                                                // TODO not sure how to support non-sol
                                                // swaps yet
                                                // also does not return the mint token properly
                                                constants::SOLANA_PROGRAM_ID
                                                    .to_string()
                                            };
                                            let amount = parsed_ix.parsed
                                                ["info"]["amount"]
                                                .to_string();
                                            res.push(TokenTransfer {
                                                amount,
                                                mint,
                                            });
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
                OptionSerializer::None | OptionSerializer::Skip => (),
            }
        }

        Ok(res)
    }

    pub fn parse_instructions(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Vec<UiInstruction>, Box<dyn std::error::Error>> {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Parsed(msg) => Ok(msg.instructions.clone()),
                UiMessage::Raw(_) => Err("Raw message not supported".into()),
            },
            _ => Err("Only EncodedTransaction::Json txs are supported".into()),
        }
    }
}
