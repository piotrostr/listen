use crate::util::healthz;
use crate::{buyer, provider::Provider, util::env};
use actix_web::post;
use actix_web::web::Json;
use actix_web::{App, Error, HttpResponse, HttpServer};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::EncodableKey;

#[derive(Deserialize, Serialize)]
pub struct BuyRequest {
    pub amm_pool: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
}

#[post("/buy")]
async fn handle_buy(buy_request: Json<BuyRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling buy req {}",
        serde_json::to_string_pretty(&buy_request)?
    );
    let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read fund keypair");
    match buyer::buy(
        &buy_request.amm_pool,
        &buy_request.input_mint,
        &buy_request.output_mint,
        buy_request.amount,
        &wallet,
        &Provider::new(env("RPC_URL").to_string()),
    )
    .await
    {
        Ok(_) => {
            info!("OK");
            Ok(HttpResponse::Ok().json(json!({"status": "OK"})))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("{}", e))),
    }
}

pub async fn run_buyer_service() -> std::io::Result<()> {
    info!("Running buyer service on 8080");
    HttpServer::new(move || App::new().service(handle_buy).service(healthz))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
