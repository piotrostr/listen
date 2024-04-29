use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
        RpcProgramAccountsConfig,
    },
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{str::FromStr, time::Duration};

pub fn must_get_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => panic!("{} env var not set", key),
    }
}

const RAYDIUM_LIQUIDITY_POOL_PUBKEY: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let signer = rpc_client.get_identity()?;

    println!("Connecting to blocks through {:?}", url);
    println!("Latest blockhash: {:?}", latest_blockhash);
    println!("Signer: {:?}", signer);

    Ok(rpc_client)
}

pub fn block_subscribe() -> Result<(), Box<dyn std::error::Error>> {
    // let url = must_get_env("SOLANA_WS_URL").replace("mainnet", "devnet");
    let url = String::from("https://api.mainnet-beta.solana.com");
    let raydium_pubkey = Pubkey::from_str(RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;

    let filter = RpcBlockSubscribeFilter::MentionsAccountOrProgram(raydium_pubkey.to_string());
    let config = RpcBlockSubscribeConfig::default();

    let ws_url = url.replace("https", "wss");
    let (mut subs, receiver) =
        PubsubClient::block_subscribe(ws_url.as_str(), filter, Some(config))?;

    println!("Filtering for mentions of {:?}", raydium_pubkey);

    while let Ok(block) = receiver.recv_timeout(Duration::from_secs(1)) {
        println!("Received block: {:?}", block);
    }

    subs.shutdown().unwrap();

    Ok(())
}

fn program_subscribe() -> Result<(), Box<dyn std::error::Error>> {
    let url = must_get_env("SOLANA_WS_URL");
    let raydium_pubkey = Pubkey::from_str(RAYDIUM_LIQUIDITY_POOL_PUBKEY)?;
    let mut config = RpcProgramAccountsConfig::default();
    config.account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::JsonParsed),
        data_slice: None,
        commitment: Some(CommitmentConfig::confirmed()),
        min_context_slot: None,
    };
    let (mut subs, receiver) =
        PubsubClient::program_subscribe(&url, &raydium_pubkey, Some(config))?;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(program_subscribe()?)
}
