use crate::state::ServiceState;
use crate::util::{pubkey_to_string, string_to_pubkey};
use crate::Provider;
use actix_web::{
    post,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Deserialize)]
pub struct BalanceRequest {
    #[serde(deserialize_with = "string_to_pubkey")]
    pubkey: Pubkey,
}

#[derive(Debug, Deserialize)]
pub struct TokenBalanceRequest {
    #[serde(deserialize_with = "string_to_pubkey")]
    pubkey: Pubkey,
    #[serde(deserialize_with = "string_to_pubkey")]
    mint: Pubkey,
}

#[derive(Debug, Deserialize)]
pub struct PricingRequest {
    mint: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    #[serde(serialize_with = "pubkey_to_string")]
    pubkey: Pubkey,
    balance: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenBalanceResponse {
    #[serde(serialize_with = "pubkey_to_string")]
    pubkey: Pubkey,
    #[serde(serialize_with = "pubkey_to_string")]
    mint: Pubkey,
    balance: u64,
}

#[post("/balance")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_balance(
    request: Json<BalanceRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let balance = Provider::get_balance(&state.rpc_client, &request.pubkey)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(BalanceResponse {
        pubkey: request.pubkey,
        balance,
    }))
}

#[post("/token_balance")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_token_balance(
    request: Json<TokenBalanceRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let balance = Provider::get_spl_balance(
        &state.rpc_client,
        &request.pubkey,
        &request.mint,
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(TokenBalanceResponse {
        pubkey: request.pubkey,
        mint: request.mint,
        balance,
    }))
}

#[post("/pricing")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pricing(
    request: Json<PricingRequest>,
) -> Result<HttpResponse, Error> {
    let price_data = Provider::get_pricing(&request.mint)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(price_data))
}
