use std::sync::Arc;

use crate::pump::{self, PumpBuyRequest};
use crate::util::{env, healthz};
use jito_searcher_client::get_searcher_client;
use log::info;
use tokio::sync::Mutex;

use actix_web::{post, web::Json, App, Error, HttpResponse, HttpServer};
use serde_json::json;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::EncodableKey;

#[post("/pump-buy")]
pub async fn handle_pump_buy(
    pump_buy_request: Json<PumpBuyRequest>,
) -> Result<HttpResponse, Error> {
    info!(
        "handling pump buy req {}",
        serde_json::to_string_pretty(&pump_buy_request)?
    );
    let pump_buy_request = pump_buy_request.clone();
    let mint = pump_buy_request.mint;
    tokio::spawn(async move {
        let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
            .expect("read fund keypair");

        let auth = Arc::new(
            Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
        );
        let mut searcher_client = Arc::new(Mutex::new(
            get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                .await
                .expect("makes searcher client"),
        ));

        let lamports = 100_000;
        pump::instabuy_pump_token(
            &wallet,
            lamports,
            &mut searcher_client,
            pump_buy_request,
        )
        .await
        .expect("pump instabuy");
    });

    Ok(HttpResponse::Ok().json(json!({
    "status": format!(
        "OK, trigerred buy of {}",
        mint.to_string())
    })))
}

pub async fn run_pump_service() -> std::io::Result<()> {
    info!("Running pump service on 6969");
    HttpServer::new(move || {
        App::new().service(handle_pump_buy).service(healthz)
    })
    .bind(("0.0.0.0", 6969))?
    .run()
    .await
}
