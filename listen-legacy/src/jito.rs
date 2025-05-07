use std::time::Duration;

use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
use jito_protos::searcher::{
    NextScheduledLeaderRequest, SubscribeBundleResultsRequest,
};
use jito_searcher_client::token_authenticator::ClientInterceptor;
use jito_searcher_client::{
    send_bundle_no_wait, send_bundle_with_confirmation,
};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    instruction::Instruction, transaction::VersionedTransaction,
};
use solana_transaction_status::{
    Encodable, EncodedTransaction, UiTransactionEncoding,
};
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::constants;

pub type SearcherClient =
    SearcherServiceClient<InterceptedService<Channel, ClientInterceptor>>;

pub async fn wait_leader(
    searcher_client: &mut SearcherClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut is_leader_slot = false;
    while !is_leader_slot {
        let next_leader = searcher_client
            .get_next_scheduled_leader(NextScheduledLeaderRequest {
                regions: vec![],
            })
            .await
            .expect("gets next scheduled leader")
            .into_inner();
        let num_slots =
            next_leader.next_leader_slot - next_leader.current_slot;
        // give three slots for calc and bundle creation
        is_leader_slot = num_slots <= 3;
        info!(
            "next jito leader slot in {num_slots} slots in {}",
            next_leader.next_leader_region
        );
        if num_slots > 50 {
            error!("next leader slot too far in the future");
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Ok(true)
}

#[timed::timed(duration(printer = "info!"))]
pub async fn send_swap_tx(
    ixs: &mut Vec<Instruction>,
    tip: u64,
    payer: &Keypair,
    searcher_client: &mut SearcherClient,
    rpc_client: &RpcClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bundle_results_subscription = searcher_client
        .subscribe_bundle_results(SubscribeBundleResultsRequest {})
        .await
        .expect("subscribe to bundle results")
        .into_inner();
    // build + sign the transactions
    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get blockhash");

    // push tip ix
    ixs.push(transfer(&payer.pubkey(), &constants::JITO_TIP_PUBKEY, tip));

    let swap_tx =
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            ixs.as_slice(),
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ));

    send_bundle_with_confirmation(
        &[swap_tx],
        rpc_client,
        searcher_client,
        &mut bundle_results_subscription,
    )
    .await
}

#[timed::timed(duration(printer = "info!"))]
pub async fn send_swap_tx_no_wait(
    ixs: &mut Vec<Instruction>,
    tip: u64,
    payer: &Keypair,
    searcher_client: &mut SearcherClient,
    rpc_client: &RpcClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get blockhash");

    ixs.push(transfer(&payer.pubkey(), &constants::JITO_TIP_PUBKEY, tip));

    let swap_tx =
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            ixs.as_slice(),
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ));

    let res = send_bundle_no_wait(&[swap_tx], searcher_client).await?;

    info!("Bundle sent. UUID: {}", res.into_inner().uuid);

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct JitoResponse {
    pub jsonrpc: String,
    pub result: String,
    pub id: i64,
}

#[timed::timed(duration(printer = "info!"))]
pub async fn send_jito_tx(
    tx: Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let encoded_tx = match tx.encode(UiTransactionEncoding::Binary) {
        EncodedTransaction::LegacyBinary(b) => b,
        _ => return Err("Failed to encode transaction".into()),
    };

    let res = client
        .post("https://mainnet.block-engine.jito.wtf/api/v1/transactions")
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded_tx]
        }))
        .send()
        .await
        .expect("send tx");

    let jito_response = res.json::<JitoResponse>().await?;

    info!("sent jito tx: {}", jito_response.result);

    Ok(jito_response.result)
}

#[cfg(test)]
mod tests {
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{
        message::Message,
        signature::Keypair,
        signer::{EncodableKey, Signer},
        system_instruction,
        transaction::Transaction,
    };

    use crate::util::env;

    #[tokio::test]
    async fn test_send_jito_tx() {
        dotenvy::dotenv().ok();
        let rpc_client = RpcClient::new(env("RPC_URL"));

        let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
            .expect("Failed to read keypair");
        let instruction = system_instruction::transfer(
            &keypair.pubkey(),
            &keypair.pubkey(),
            1_000_000, // lamports (0.001 SOL)
        );

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("Failed to get recent blockhash");

        let message = Message::new(&[instruction], Some(&keypair.pubkey()));
        let tx = Transaction::new(&[&keypair], message, recent_blockhash);

        super::send_jito_tx(tx).await.unwrap();
    }
}
