use crate::state::ServiceState;
use crate::util::{pubkey_to_string, string_to_pubkey};
use actix_web::{
    post,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuyRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    input_mint: Pubkey,

    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    output_mint: Pubkey,

    /// slippage in bps
    slippage: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    mint: Pubkey,

    /// token_amount
    token_amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[post("/buy")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_buy(
    buy_request: Json<BuyRequest>,
    _state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "buy_request": buy_request.into_inner(),
    })))
}

#[post("/sell")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_sell(
    sell_request: Json<SellRequest>,
    _state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "sell_request": sell_request.into_inner(),
    })))
}
