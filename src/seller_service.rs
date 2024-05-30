use crate::seller;
use crate::util::healthz;
use crate::{
    buyer,
    provider::Provider,
    util::{env, pubkey_to_string, string_to_pubkey},
};
use actix_web::post;
use actix_web::web::Json;
use actix_web::{App, Error, HttpResponse, HttpServer};
use base64::Engine;
use futures_util::StreamExt;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};

#[derive(Deserialize, Serialize)]
pub struct SellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub input_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub output_mint: Pubkey,
    pub lamports_spent: u64,
}

#[post("/sell")]
async fn handle_sell(sell_request: Json<SellRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling sell_request {}",
        serde_json::to_string_pretty(&sell_request)?
    );
    tokio::spawn(async move {
        let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
        let provider = Provider::new(env("RPC_URL"));
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &sell_request.input_mint,
        );
        let pubsub_client = PubsubClient::new(&env("WS_URL"))
            .await
            .expect("make pubsub client");
        let balance = match provider
            .rpc_client
            .get_token_account_balance(&token_account)
            .await
        {
            Ok(balance) => balance
                .amount
                .parse::<u64>()
                .expect("balance string to u64"),
            Err(e) => {
                warn!("error getting balance: {}", e);
                info!("listening on token account {}", token_account.to_string());
                get_spl_balance_stream(&pubsub_client, &token_account)
                    .await
                    .expect("get_spl_balance_stream")
            }
        };
        info!("balance: {}", balance);
        let ok = seller::listen_price(
            &sell_request.amm_pool,
            &provider.rpc_client,
            &pubsub_client,
            Some(balance),
            Some(sell_request.lamports_spent as f64 * 1.6),
            Some(sell_request.lamports_spent as f64 * 0.8),
            Some(sell_request.lamports_spent),
        )
        .await
        .expect("listen price");
        if !ok {
            return;
        }

        buyer::swap(
            &sell_request.amm_pool,
            &sell_request.input_mint,
            &sell_request.output_mint,
            balance,
            &wallet,
            &provider,
        )
        .await
        .expect("buy");
    });

    Ok(HttpResponse::Ok().json(json!({"status": "OK, triggered sell"})))
}

pub async fn get_spl_balance_stream(
    pubsub_client: &PubsubClient,
    token_account: &Pubkey,
) -> Result<u64, Box<dyn std::error::Error>> {
    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            token_account,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await
        .expect("account_subscribe");

    tokio::select! {
        log = stream.next() => {
            if let UiAccountData::Binary(data, UiAccountEncoding::Base64) = log.expect("log").value.data {
                let log_data = base64::prelude::BASE64_STANDARD.decode(&data).expect("decode spl b64");
                let spl_account = spl_token::state::Account::unpack(&log_data).expect("unpack spl");
                unsub().await;
                Ok(spl_account.amount)
            } else {
                warn!("get_spl_balance_stream {}: unexpected data", token_account.to_string());
                Err("unexpected data".into())
            }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
            warn!("get_spl_balance_stream {}: timeout", token_account.to_string());
            Err("timeout".into())
        },
    }
}

pub async fn run_seller_service() -> std::io::Result<()> {
    info!("Running seller service on 8081");
    HttpServer::new(move || App::new().service(handle_sell).service(healthz))
        .bind(("0.0.0.0", 8081))?
        .run()
        .await
}
