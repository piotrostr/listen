use crate::{collector, constants, util::env};
use actix_web::{
    error, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use futures_util::StreamExt;
use log::{info, warn};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

pub async fn run_listener_service() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        println!("{}", env("WS_URL"));
        let collector = Arc::new(collector::new().await.expect("collector"));
        let client = PubsubClient::new(&env("WS_URL"))
            .await
            .expect("pubsub client async");
        let (mut notifications, unsub) = client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![
                    constants::FEE_PROGRAM_ID.to_string(),
                ]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::processed()),
                },
            )
            .await
            .expect("subscribe to logs");
        info!("Listening for LP events");
        while let Some(log) = notifications.next().await {
            println!("{:?}", log);
            let collector = Arc::clone(&collector);
            if log.value.err.is_none() {
                // tx.send(log).await.expect("send log");
                tokio::spawn(async move {
                    for _ in 0..3 {
                        info!("passing log {}", log.value.signature);
                        match reqwest::get(format!(
                            "http://localhost:8080/new_pair/{}",
                            log.value.signature
                        ))
                        .await
                        {
                            Ok(res) => {
                                info!("response: {:?}", res);
                                if res.status().is_server_error() {
                                    warn!("server error");
                                    tokio::time::sleep(
                                        tokio::time::Duration::from_secs(5),
                                    )
                                    .await;
                                    continue;
                                }
                                let result = res
                                    .json::<serde_json::Value>()
                                    .await
                                    .unwrap();
                                info!(
                                    "result: {}",
                                    serde_json::to_string_pretty(&result)
                                        .unwrap()
                                );
                                let inserted_id = collector
                                    .insert_generic(result)
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
    req: HttpRequest,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    dbg!(req);
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
    info!("{}", serde_json::to_string_pretty(&obj).unwrap());

    // reqwest::get(format!(
    //     "http://localhost:8080/signature/{}",
    //     obj["signature"].as_str().unwrap()
    // ))
    // .await
    // .unwrap();

    // body is loaded, now we can deserialize serde-json
    Ok(HttpResponse::Ok().body("ok"))
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("im ok")
}

pub async fn run_listener_webhook_service() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(handle_webhook).service(healthz))
        .bind(("127.0.0.1", 8079))?
        .run()
        .await
}
