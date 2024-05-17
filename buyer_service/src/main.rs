use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv_codegen::dotenv;
use jito_searcher_client::get_searcher_client;
use listen::tx_parser;
use listen::{buyer, Provider};
use log::info;
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
use std::sync::Arc;

use flexi_logger::{Duplicate, FileSpec, Logger, WriteMode};

pub async fn get_tx_async(
    signature: &str,
) -> Result<EncodedConfirmedTransactionWithStatusMeta, Box<dyn std::error::Error>>
{
    let rpc_client = RpcClient::new(dotenv!("RPC_URL").to_string());
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
    let signature = signature.into_inner();
    let auth = Keypair::read_from_file(dotenv!("AUTH_KEYPAIR_PATH"))
        .expect("read auth keypair");
    let mut searcher_client =
        get_searcher_client(dotenv!("BLOCK_ENGINE_URL"), &Arc::new(auth))
            .await
            .expect("makes searcher client");
    let provider = Provider::new(dotenv!("RPC_URL").to_string());
    let wallet = Keypair::read_from_file(dotenv!("FUND_KEYPAIR_PATH"))
        .expect("read fund keypair");
    let txn = provider.get_tx(&signature).await.unwrap();
    let new_pool_info =
        tx_parser::parse_new_pool(&txn).expect("parse pool info");
    info!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "slot": txn.slot,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "input": new_pool_info.input_mint.to_string(),
            "output": new_pool_info.output_mint.to_string(),
            "pool": new_pool_info.amm_pool_id.to_string(),
            "amm_pool": new_pool_info.amm_pool_id.to_string(),
        }))
        .expect("serialize pool info")
    );
    match buyer::handle_new_pair(
        new_pool_info,
        10000000,
        2000,
        &wallet,
        &provider,
        &mut searcher_client,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _logger = Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::Async)
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .route("/new_pair/{signature}", web::get().to(handle_new_pair))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
