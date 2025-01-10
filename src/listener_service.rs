use crate::{
    checker::PoolAccounts,
    checker_service::ChecksRequest,
    collector, constants,
    http_client::HttpClient,
    util::{env, healthz},
};
use actix_web::{
    error, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use futures_util::StreamExt;
use log::{debug, info, warn};
use serde_json::Value;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{str::FromStr, sync::Arc};

pub async fn run_listener_pubsub_service(
) -> Result<(), Box<dyn std::error::Error>> {
    info!("{}", env("WS_URL"));
    tokio::spawn(async move {
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
            debug!("{:?}", log);
            let collector = Arc::clone(&collector);
            if log.value.err.is_none() {
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
async fn receive_webhook(
    req: HttpRequest,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    debug!("webhook request: {:?}", req);
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

    let data = serde_json::from_slice::<Value>(&body)?
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    debug!("{}", serde_json::to_string_pretty(&data).unwrap());

    if !data["instructions"].is_array() {
        return Ok(HttpResponse::BadRequest().body("instructions not array"));
    }

    tokio::spawn(async move {
        reqwest::Client::new()
            .post(env("LISTENER_URL") + "/handler")
            .json(&data)
            .send()
            .await
            .expect("pass on");
    });

    Ok(HttpResponse::Ok().finish())
}

#[post("/handler")]
async fn handle_webhook(
    data: web::Json<Value>,
) -> Result<HttpResponse, Error> {
    let signature = data["signature"].as_str().unwrap().to_string();
    info!("handling webhook {}", signature);

    for instruction in data["instructions"].as_array().unwrap() {
        let accounts = instruction["accounts"].as_array().unwrap();
        if accounts.len() == 21 {
            info!("found LP instruction");
            let amm_pool =
                Pubkey::from_str(accounts[4].as_str().unwrap()).unwrap();
            let lp_mint =
                Pubkey::from_str(accounts[7].as_str().unwrap()).unwrap();
            let coin_mint =
                Pubkey::from_str(accounts[8].as_str().unwrap()).unwrap();
            let pc_mint =
                Pubkey::from_str(accounts[9].as_str().unwrap()).unwrap();
            let pool_coin_token_account =
                Pubkey::from_str(accounts[10].as_str().unwrap()).unwrap();
            let pool_pc_token_account =
                Pubkey::from_str(accounts[11].as_str().unwrap()).unwrap();
            let user_wallet =
                Pubkey::from_str(accounts[17].as_str().unwrap()).unwrap();
            let user_token_coin =
                Pubkey::from_str(accounts[18].as_str().unwrap()).unwrap();
            let user_token_pc =
                Pubkey::from_str(accounts[19].as_str().unwrap()).unwrap();
            let user_lp_token =
                Pubkey::from_str(accounts[20].as_str().unwrap()).unwrap();
            let pool_accounts = PoolAccounts {
                amm_pool,
                lp_mint,
                coin_mint,
                pc_mint,
                pool_coin_token_account,
                pool_pc_token_account,
                user_wallet,
                user_token_coin,
                user_token_pc,
                user_lp_token,
            };
            let transfers = data["tokenTransfers"].as_array().unwrap();
            let initial_sol_pooled = transfers
                .iter()
                .find(|transfer| {
                    transfer["mint"].as_str().unwrap()
                        == constants::SOLANA_PROGRAM_ID.to_string()
                })
                .expect("find sol mint")["tokenAmount"]
                .as_f64()
                .unwrap();
            let initial_token_pooled = transfers
                .iter()
                .find(|transfer| {
                    let mint = transfer["mint"].as_str().unwrap();
                    mint != constants::SOLANA_PROGRAM_ID.to_string()
                        && mint != pool_accounts.lp_mint.to_string()
                })
                .expect("find token mint")["tokenAmount"]
                .as_f64()
                .unwrap();
            let checks_request = ChecksRequest {
                signature: signature.clone(),
                accounts: pool_accounts,
                slot: data["slot"].as_u64().unwrap(),
                initial_sol_pooled,
                initial_token_pooled,
            };
            tokio::spawn(async move {
                HttpClient::new()
                    .checks(&checks_request)
                    .await
                    .expect("checks");
            });
        }
    }

    // body is loaded, now we can deserialize serde-json
    Ok(HttpResponse::Ok().body("ok"))
}

pub async fn run_listener_webhook_service() -> std::io::Result<()> {
    info!("Running listener service (webhooks) on 8078");
    HttpServer::new(|| {
        App::new()
            .service(receive_webhook)
            .service(handle_webhook)
            .service(healthz)
    })
    .bind(("0.0.0.0", 8078))?
    .run()
    .await
}
