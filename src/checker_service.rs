use crate::{
    buyer_service::BuyRequest,
    checker::{Checklist, PoolAccounts, _run_checks},
    constants,
    http_client::HttpClient,
    util::{env, healthz},
};
use actix_web::web::Json;
use actix_web::{post, App, Error, HttpResponse, HttpServer, Result};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;

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
pub async fn handle_checks(
    checks_request: Json<ChecksRequest>,
) -> Result<HttpResponse, Error> {
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
    let (ok, checklist) = match _run_checks(
        &rpc_client,
        checks_request.accounts,
        checks_request.slot,
        true,
    )
    .await
    {
        Ok((ok, checklist)) => (ok, checklist),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(
                json!({"error": format!("Error running checks: {}", e)}),
            ));
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

    let amm_pool = checks_request.accounts.amm_pool;
    let input_mint = constants::SOLANA_PROGRAM_ID;
    tokio::spawn(async move {
        let amount = if token_result.checklist.is_pump_fun {
            50_000_000
        } else {
            // pass on non-pumps
            return;
        };
        HttpClient::new()
            .buy(&BuyRequest {
                amm_pool,
                input_mint,
                output_mint,
                amount,
            })
            .await
            .expect("buy");
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
