use crate::constants;
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use log::{info, warn};
use solana_client::{
    nonblocking,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;

pub async fn run_listener() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        let client =
            nonblocking::pubsub_client::PubsubClient::new(dotenv!("WS_URL"))
                .await
                .expect("pubsub client async");
        let (mut notifications, unsub) = client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![
                    constants::FEE_PROGRAM_ID.to_string(),
                ]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )
            .await
            .expect("subscribe to logs");
        info!("Listening for LP events");
        while let Some(log) = notifications.next().await {
            if log.value.err.is_none() {
                info!("passing log {}", log.value.signature);
                // tx.send(log).await.expect("send log");
                tokio::spawn(async move {
                    match reqwest::get(format!(
                        "http://localhost:8080/new_pair/{}",
                        log.value.signature
                    ))
                    .await
                    {
                        Ok(_) => info!("ok"),
                        Err(e) => warn!("error sending log: {}", e),
                    };
                });
            }
        }
        unsub().await;
    })
    .await?;

    Ok(())
}
