use std::str::FromStr;

use crate::{raydium::Holding, state::ServiceState};
use crate::Provider;
use actix_web::{
    post, get,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BalanceRequest {
    pubkey: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    pubkey: String,
    balance: u64,
}

#[utoipa::path(
    post, 
    path = "/balance",
    responses((status = 200, body = BalanceResponse)),
    tag = "balance"
)]
#[post("/balance")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_balance(
    request: Json<BalanceRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let pubkey = Pubkey::from_str(&request.pubkey)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let balance = Provider::get_balance(&state.rpc_client, &pubkey)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(BalanceResponse {
        pubkey: pubkey.to_string(),
        balance,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PubkeyResponse {
    pubkey: String,
}

#[utoipa::path(
    get, 
    path = "/pubkey",
    responses((status = 200, body = PubkeyResponse)),
    tag = "balance"
)]
#[get("/pubkey")]
pub async fn handle_get_pubkey(
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let pubkey = state.wallet.lock().await.pubkey();
    Ok(HttpResponse::Ok().json(PubkeyResponse {
        pubkey: pubkey.to_string(),
    }))
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenBalanceRequest {
    pubkey: String,
    mint: String,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct TokenBalanceResponse {
    pubkey: String,
    mint: String,
    balance: u64,
}

#[utoipa::path(
    post, 
    path = "/token_balance", 
    responses((status = 200, body = TokenBalanceResponse)),
    tag = "balance"
)]
#[post("/token_balance")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_token_balance(
    request: Json<TokenBalanceRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let pubkey = Pubkey::from_str(&request.pubkey)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let mint = Pubkey::from_str(&request.mint)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let balance = Provider::get_spl_balance(&state.rpc_client, &pubkey, &mint)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(TokenBalanceResponse {
        pubkey: pubkey.to_string(),
        mint: mint.to_string(),
        balance,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PriceRequest {
    mint: String,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct PriceResponse {
    mint: String,
    price: f64,
}

#[utoipa::path(
    post, 
    path = "/price", 
    responses((status = 200, body = PriceResponse)),
    request_body = PriceRequest,
    description = "Get the price of a token",
    tag = "token"
)]
#[post("/price")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pricing(
    request: Json<PriceRequest>,
) -> Result<HttpResponse, Error> {
    let price_data = Provider::get_pricing(&request.mint)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(price) = price_data.data.get(&request.mint) {
        return Ok(HttpResponse::Ok().json(PriceResponse {
            mint: request.mint.clone(),
            price: price.price,
        }));
    }; 

    Ok(HttpResponse::NotFound().finish())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HoldingsResponse {
    pub holdings: Vec<Holding>,
}

#[utoipa::path(
    get, 
    path = "/holdings", 
    responses((status = 200, body = HoldingsResponse)),
    tag = "balance"
)]
#[get("/holdings")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_get_holdings(state: Data<ServiceState>) -> Result<HttpResponse, Error> {
    let holdings = Provider::get_holdings(
        &state.rpc_client, 
        &state.wallet.lock().await.pubkey()
    )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(HoldingsResponse { holdings}))
}
