use crate::seller::get_sol_pooled_vault;
use crate::seller_service::SellRequest;
use crate::util::healthz;
use crate::{
    buyer,
    provider::Provider,
    util::{env, pubkey_to_string, string_to_pubkey},
};
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
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub input_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub output_mint: Pubkey,
    pub sol_vault: Pubkey,
    pub amount: u64,
}

#[post("/buy")]
async fn handle_buy(buy_request: Json<BuyRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling buy req {}",
        serde_json::to_string_pretty(&buy_request)?
    );
    tokio::spawn(async move {
        let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read fund keypair");
        let provider = &Provider::new(env("RPC_URL").to_string());
        buyer::buy(
            &buy_request.amm_pool,
            &buy_request.input_mint,
            &buy_request.output_mint,
            buy_request.amount,
            &wallet,
            provider,
        )
        .await
        .expect("buy");
        let sol_pooled_when_bought =
            get_sol_pooled_vault(&buy_request.sol_vault, &provider.rpc_client).await;
        reqwest::Client::new()
            .post(env("SELLER_URL") + "/sell")
            .json(&SellRequest {
                amm_pool: buy_request.amm_pool,
                input_mint: buy_request.output_mint,
                output_mint: buy_request.input_mint,
                sol_vault: buy_request.sol_vault,
                sol_pooled_when_bought,
            })
            .send()
            .await
            .expect("send sell request");
    });

    Ok(HttpResponse::Ok().json(json!({"status": "OK, trigerred buy"})))
}

pub async fn run_buyer_service() -> std::io::Result<()> {
    info!("Running buyer service on 8080");
    HttpServer::new(move || App::new().service(handle_buy).service(healthz))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
