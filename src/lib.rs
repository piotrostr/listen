pub mod types;

use core::panic;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
        RpcProgramAccountsConfig, RpcTransactionConfig,
        RpcTransactionLogsConfig, RpcTransactionLogsFilter,
    },
};
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction, UiTransactionEncoding,
};
use std::{str::FromStr, time::Duration};

pub struct Listener {
    pub url: String,
    pub ws_url: String,
    pub rpc_client: RpcClient,
}

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client =
        RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let signer = rpc_client.get_identity()?;

    println!("Connecting to blocks through {:?}", url);
    println!("Latest blockhash: {:?}", latest_blockhash);
    println!("Signer: {:?}", signer);

    Ok(rpc_client)
}

impl Listener {
    pub fn block_subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        // let url = must_get_env("SOLANA_WS_URL").replace("mainnet", "devnet");
        let raydium_pubkey = Pubkey::from_str(RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;

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
        let raydium_pubkey = Pubkey::from_str(RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;
        let mut config = RpcProgramAccountsConfig::default();
        config.account_config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::JsonParsed),
            data_slice: None,
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: None,
        };
        let (mut subs, receiver) = PubsubClient::program_subscribe(
            &self.ws_url,
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
        // let url = must_get_env("SOLANA_WS_URL");
        let raydium_pubkey = Pubkey::from_str(RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };
        let filter =
            RpcTransactionLogsFilter::Mentions(
                vec![raydium_pubkey.to_string()],
            );
        let (mut subs, receiver) =
            PubsubClient::logs_subscribe(&self.url, filter, config)?;

        println!("Connecting to logs for {:?}", raydium_pubkey);

        if let Ok(log) = receiver.recv_timeout(Duration::from_secs(1)) {
            println!("{}", serde_json::to_string_pretty(&log)?);
        }

        subs.shutdown().unwrap();

        Ok(())
    }

    pub fn get_tx(
        &self,
        signature: &str,
    ) -> Result<
        EncodedConfirmedTransactionWithStatusMeta,
        Box<dyn std::error::Error>,
    > {
        let sig = Signature::from_str(signature)?;
        let tx = self.rpc_client.get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )?;
        Ok(tx)
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
                UiMessage::Raw(_) => panic!("Unsupported raw message"),
            },
            _ => panic!("Unsupported transaction encoding"),
        }
    }

    pub async fn get_pricing(
        &self,
        mint: &str,
    ) -> Result<types::PriceResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://price.jup.ag/v4/price?ids={}&vsToken={}",
            mint, SOLANA_PROGRAM_ID,
        );
        println!("Getting pricing from: {:?}", url);
        let client = reqwest::Client::new();
        let res = client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await?;
        let data = res.json::<types::PriceResponse>().await?;
        Ok(data)
    }

    pub async fn get_notional(
        &self,
        signature: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = Signature::from_str(signature)?;
        let _ = self.get_tx(signature)?;
        Ok(())
    }
}

pub const SOLANA_PROGRAM_ID: &str =
    "So11111111111111111111111111111111111111112";

pub const RAYDIUM_LIQUIDITY_POOL_PUBKEY: &str =
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Listener {
        let listener = Listener {
            url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            rpc_client: get_client("https://api.mainnet-beta.solana.com")
                .unwrap(),
        };
        listener
    }

    #[test]
    fn test_get_pricing() {
        let listener = setup();
        let mint = "Fv17uvL3nsD4tBJaowdKz9SUsKFoxeZdcTuGTaKgyYQU";

        let pricing = tokio_test::block_on(listener.get_pricing(mint)).unwrap();
        assert!(pricing.data[mint].price > 0., "Price not found");
    }
}
