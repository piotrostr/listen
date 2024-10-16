use futures_util::StreamExt;
use solana_account_decoder::UiAccountEncoding;
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;
use std::str::FromStr;

use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::pubkey::Pubkey;

fn must_get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{} env var not set", key))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mint =
        Pubkey::from_str("412Mr1t8g1xSzW4wBaCV8J8KDFrhff46aNqGMSoK1asL")?;
    let pubsub_client = PubsubClient::new(&must_get_env("WS_URL")).await?;
    let (mut rec, unsub) = pubsub_client
        .account_subscribe(
            &mint,
            Some(RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            }),
        )
        .await?;
    while let Some(message) = rec.next().await {
        println!("{:?}", message);
    }

    if (tokio::signal::ctrl_c().await).is_ok() {
        println!("Ctrl-C received, shutting down");
        unsub().await;
    }

    Ok(())
}
