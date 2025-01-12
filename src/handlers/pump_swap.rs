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
pub struct PumpBuyRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    input_mint: Pubkey,

    /// token_amount to buy
    token_amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PumpSellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    input_mint: Pubkey,

    /// token_amount to sell
    token_amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[post("/pump/buy")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pump_buy(
    pump_buy_request: Json<PumpBuyRequest>,
    _state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "pump_buy_request": pump_buy_request.into_inner(),
    })))
}

#[post("/pump/sell")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pump_sell(
    pump_sell_request: Json<PumpSellRequest>,
    _state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "pump_sell_request": pump_sell_request.into_inner(),
    })))
}
