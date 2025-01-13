use crate::http_client::HttpClient;
use crate::seller_service::SellRequest;
use crate::util::healthz;
use crate::{
    buyer,
    util::{env, pubkey_to_string, string_to_pubkey},
};
use actix_web::post;
use actix_web::web::Json;
use actix_web::{App, Error, HttpResponse, HttpServer};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
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
    pub amount: u64,
}

#[post("/buy")]
async fn handle_buy(
    buy_request: Json<BuyRequest>,
) -> Result<HttpResponse, Error> {
    info!(
        "handling buy req {}",
        serde_json::to_string_pretty(&buy_request)?
    );
    let mint = buy_request.output_mint;
    tokio::spawn(async move {
        let Ok(wallet) = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
        else {
            error!("Failed to read wallet");
            return;
        };
        let rpc_client = RpcClient::new(env("RPC_URL"));
        if let Err(e) = buyer::swap(
            &buy_request.amm_pool,
            &buy_request.input_mint,
            &buy_request.output_mint,
            buy_request.amount,
            &wallet,
            &rpc_client,
        )
        .await
        {
            error!("could not swap {e}, exiting");
            return;
        };

        if let Err(e) = HttpClient::new()
            .sell(&SellRequest {
                amm_pool: buy_request.amm_pool,
                input_mint: buy_request.output_mint,
                output_mint: buy_request.input_mint,
                lamports_spent: buy_request.amount,
                insta: None,
            })
            .await
        {
            error!("could not sell {e}, exiting");
        }
    });

    Ok(HttpResponse::Ok()
        .json(json!({"status": format!("OK, trigerred buy of {}", mint.to_string())})))
}

pub struct BalanceContext {
    pub lamports: u64,
}

pub async fn run_buyer_service() -> std::io::Result<()> {
    info!("Running buyer service on 8080");
    HttpServer::new(move || App::new().service(handle_buy).service(healthz))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
