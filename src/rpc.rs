use log::info;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::{EncodableKey, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

pub fn eval_rpc(rpc_url: &str) {
    info!("Evaluating RPC: {}", rpc_url);
    let rpc_client = RpcClient::new(rpc_url);
    let lamports = 10000u64;
    let wallet =
        Keypair::read_from_file(std::env::var("HOME").unwrap() + "/.config/solana/id.json")
            .unwrap();
    info!("signer: {}", wallet.pubkey());
    let instructions = [solana_sdk::system_instruction::transfer(
        &wallet.pubkey(),
        &Pubkey::from_str("9SDTi7rzsCt1Y1QDY4n6NvFHn33tcYoVYfoEuXR7dQEM").unwrap(),
        lamports,
    )];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &[&wallet],
        rpc_client.get_latest_blockhash().unwrap(),
    );
    let slot = rpc_client.get_slot().unwrap();
    println!("Slot: {}", slot);
    println!("{:?}", rpc_client.simulate_transaction(&transaction));
    let signature = rpc_client
        .send_transaction_with_config(
            &transaction,
            RpcSendTransactionConfig {
                encoding: Some(solana_transaction_status::UiTransactionEncoding::Base58),
                skip_preflight: true,
                preflight_commitment: Some(CommitmentLevel::Processed),
                max_retries: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
    println!("Signature: {}", signature);

    wait_tx(&rpc_client, &signature);
}

pub fn wait_tx(rpc_client: &RpcClient, signature: &Signature) {
    loop {

        match rpc_client.get_transaction_with_config(
            signature,
            RpcTransactionConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: None,
                ..Default::default()
            },
        ) {
            Err(e) => {
                println!("{:?}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Ok(tx) => { println!("{:?}", tx) }
        }
    }
}
