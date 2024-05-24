use crate::checker::run_checks;
use crate::constants;
use crate::{buyer, provider::Provider};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv_codegen::dotenv;
use log::info;
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
    let mut token_result = buyer::TokenResult::default();
    token_result.creation_signature = signature.clone();
    token_result.timestamp_received = chrono::Utc::now().to_rfc3339();
    let signature = signature.into_inner();
    // TODO match statement this
    let (ok, checklist) = run_checks(signature).await.expect("checks");
    token_result.checklist = checklist;
    if !ok {
        info!("{} Not OK", token_result.checklist.mint.to_string());
        return HttpResponse::Ok().json(token_result);
    }
    let wallet = Keypair::read_from_file(dotenv!("FUND_KEYPAIR_PATH"))
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
        50_000_000, // 0.05 sol, no rugs with the new method
        &wallet,
        &Provider::new(dotenv!("RPC_URL").to_string()),
    )
    .await
    {
        Ok(_) => {
            info!("OK");
            token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
            HttpResponse::Ok().json(token_result)
        }
        Err(e) => {
            token_result.timestamp_finalized = chrono::Utc::now().to_rfc3339();
            token_result.error = Some(format!("{}", e));
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

pub async fn run_buyer_service() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .route("/new_pair/{signature}", web::get().to(handle_new_pair))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
