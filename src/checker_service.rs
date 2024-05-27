use std::str::FromStr;

use crate::{
    buyer_service::BuyRequest,
    checker::{Checklist, PoolAccounts, _run_checks},
    constants,
    util::{env, healthz},
};
use actix_web::web::Json;
use actix_web::{post, App, Error, HttpResponse, HttpServer, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize, Serialize)]
pub struct ChecksRequest {
    pub signature: String,
    pub accounts: PoolAccounts,
    pub slot: u64,
    pub initial_sol_pooled: f64,
    pub initial_token_pooled: f64,
}

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct TokenResult {
    pub creation_signature: String,
    pub timestamp_received: String,
    pub timestamp_finalized: String,
    pub checklist: Checklist,
}

#[post("/checks")]
pub async fn handle_checks(checks_request: Json<ChecksRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling checks request {}",
        serde_json::to_string_pretty(&checks_request)?
    );
    let mut token_result = TokenResult {
        creation_signature: checks_request.signature.clone(),
        timestamp_received: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let rpc_client = RpcClient::new(env("RPC_URL"));
    let (ok, checklist) =
        match _run_checks(&rpc_client, checks_request.accounts, checks_request.slot).await {
            Ok((ok, checklist)) => (ok, checklist),
            Err(e) => {
                return Ok(HttpResponse::InternalServerError()
                    .json(json!({ "error": format!("Error running checks: {}", e)})));
            }
        };
    let output_mint = checklist.mint;
    token_result.checklist = checklist;
    if !ok {
        info!("{} Not OK", token_result.checklist.mint.to_string());
        token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
        return Ok(HttpResponse::Ok().json(token_result));
    }

    info!(
        "{} OK, sending to buyer",
        token_result.checklist.mint.to_string()
    );

    let sol_vault = if checks_request.accounts.coin_mint.to_string() == constants::SOLANA_PROGRAM_ID
    {
        checks_request.accounts.pool_coin_token_account
    } else {
        checks_request.accounts.pool_pc_token_account
    };

    let amm_pool = checks_request.accounts.amm_pool;
    let input_mint = Pubkey::from_str(constants::SOLANA_PROGRAM_ID).unwrap();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        match client
            .post(env("BUYER_URL") + "/buy")
            .json(&BuyRequest {
                amm_pool,
                input_mint,
                output_mint,
                sol_vault,
                amount: 1_000_000,
            })
            .send()
            .await
        {
            Ok(response) => {
                info!(
                    "response: {}",
                    serde_json::to_string_pretty(
                        &response.json::<serde_json::Value>().await.unwrap()
                    )
                    .unwrap()
                );
            }
            Err(e) => {
                warn!("error: {}", e);
            }
        }
    });

    token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
    Ok(HttpResponse::Ok().json(token_result))
}

pub async fn run_checker_service() -> std::io::Result<()> {
    info!("Running checker service on 8079");
    HttpServer::new(move || App::new().service(handle_checks).service(healthz))
        .bind(("0.0.0.0", 8079))?
        .run()
        .await
}
