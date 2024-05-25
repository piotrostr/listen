use crate::checker::{PoolAccounts, _run_checks};
use crate::{
    buyer, checker::run_checks, constants, provider::Provider, util::env,
};
use actix_web::web::Json;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signer::EncodableKey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};
use std::str::FromStr;

pub async fn get_tx_async(
    signature: &str,
) -> Result<EncodedConfirmedTransactionWithStatusMeta, Box<dyn std::error::Error>>
{
    let rpc_client = RpcClient::new(env("RPC_URL"));
    let sig = Signature::from_str(signature)?;
    let tx = rpc_client
        .get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(1),
            },
        )
        .await?;
    Ok(tx)
}

async fn handle_new_pair(signature: web::Path<String>) -> impl Responder {
    let mut token_result = buyer::TokenResult {
        creation_signature: signature.clone(),
        timestamp_received: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let signature = signature.into_inner();
    let (ok, checklist) = match run_checks(signature).await {
        Ok((ok, checklist)) => (ok, checklist),
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                json!({ "error": format!("Error running checks: {}", e)}),
            );
        }
    };
    token_result.checklist = checklist;
    if !ok {
        info!("{} Not OK", token_result.checklist.mint.to_string());
        return HttpResponse::Ok().json(token_result);
    }
    let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
        .expect("read fund keypair");
    let accounts = &token_result.checklist.accounts;
    let (input_mint, output_mint) =
        if accounts.coin_mint.to_string() == constants::SOLANA_PROGRAM_ID {
            (accounts.coin_mint, accounts.pc_mint)
        } else {
            (accounts.pc_mint, accounts.coin_mint)
        };
    match buyer::buy(
        &token_result.checklist.accounts.amm_pool,
        &input_mint,
        &output_mint,
        // 0.005 sol, no rugs but ppl still dump:( seller service is a
        // must-have!
        5_000_000,
        &wallet,
        &Provider::new(env("RPC_URL").to_string()),
    )
    .await
    {
        Ok(_) => {
            info!("OK");
            token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
            HttpResponse::Ok().json(token_result)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

#[derive(Deserialize, Serialize)]
pub struct ParsedPayload {
    pub signature: String,
    pub accounts: PoolAccounts,
    pub slot: u64,
}

pub async fn handle_new_pair_parsed(
    payload: Json<ParsedPayload>,
) -> impl Responder {
    info!("handling {} ({})", payload.signature, payload.slot);
    let mut token_result = buyer::TokenResult {
        creation_signature: payload.signature.clone(),
        timestamp_received: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let rpc_client = RpcClient::new(env("RPC_URL"));
    let (ok, checklist) =
        match _run_checks(&rpc_client, payload.accounts, payload.slot).await {
            Ok((ok, checklist)) => (ok, checklist),
            Err(e) => {
                return HttpResponse::InternalServerError().json(
                    json!({ "error": format!("Error running checks: {}", e)}),
                );
            }
        };
    token_result.checklist = checklist;
    if !ok {
        info!("{} Not OK", token_result.checklist.mint.to_string());
        token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
        return HttpResponse::Ok().json(token_result);
    }
    let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
        .expect("read fund keypair");
    let accounts = &token_result.checklist.accounts;
    let (input_mint, output_mint) =
        if accounts.coin_mint.to_string() == constants::SOLANA_PROGRAM_ID {
            (accounts.coin_mint, accounts.pc_mint)
        } else {
            (accounts.pc_mint, accounts.coin_mint)
        };
    match buyer::buy(
        &token_result.checklist.accounts.amm_pool,
        &input_mint,
        &output_mint,
        // 0.005 sol, no rugs but ppl still dump:( seller service is a
        // must-have!
        5_000_000,
        &wallet,
        &Provider::new(env("RPC_URL").to_string()),
    )
    .await
    {
        Ok(_) => {
            info!("OK");
            token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
            HttpResponse::Ok().json(token_result)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn run_buyer_service() -> std::io::Result<()> {
    info!("Running buyer service on 8080");
    HttpServer::new(move || {
        App::new()
            .route("/new_pair/{signature}", web::get().to(handle_new_pair))
            .route("/parsed", web::post().to(handle_new_pair_parsed))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
