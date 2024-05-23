use crate::{buyer, collector, constants};
use actix_web::{
    error, get, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use log::{info, warn};
use solana_client::{
    nonblocking,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

pub async fn run_listener_service() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let collector = Arc::new(collector::new().await.expect("collector"));
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
            let collector = Arc::clone(&collector);
            if log.value.err.is_none() {
                info!("passing log {}", log.value.signature);
                // tx.send(log).await.expect("send log");
                tokio::spawn(async move {
                    for _ in 0..3 {
                        match reqwest::get(format!(
                            "http://localhost:8080/new_pair/{}",
                            log.value.signature
                        ))
                        .await
                        {
                            Ok(res) => {
                                let token_result = res
                                    .json::<buyer::TokenResult>()
                                    .await
                                    .unwrap();
                                info!(
                                    "token result: {}",
                                    serde_json::to_string_pretty(&token_result)
                                        .unwrap()
                                );
                                let inserted_id = collector
                                    .insert(token_result)
                                    .await
                                    .expect("insert");
                                info!(
                                    "inserted id: {}",
                                    inserted_id.to_string()
                                );
                                break;
                            }
                            Err(e) => {
                                warn!("error sending log: {}", e);
                                tokio::time::sleep(
                                    tokio::time::Duration::from_secs(5),
                                )
                                .await;
                            }
                        };
                    }
                });
            }
        }
        unsub().await;
    })
    .await?;

    Ok(())
}

#[post("/")]
async fn handle_webhook(
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    info!(
        "received webhook at timestamp (unix) {}",
        chrono::Utc::now().timestamp()
    );
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    let max_size: usize = 262_144; // max payload size is 256k
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > max_size {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<serde_json::Value>(&body)?;
    println!("{}", serde_json::to_string_pretty(&obj).unwrap());

    // body is loaded, now we can deserialize serde-json
    Ok(HttpResponse::Ok().body("ok"))
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("im ok")
}

pub async fn run_listener_webhook_service() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(handle_webhook).service(healthz))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}