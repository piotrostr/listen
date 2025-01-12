use crate::jup::Jupiter;
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
pub struct SwapRequest {
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

    amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[post("/swap")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_swap(
    swap_request: Json<SwapRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let swap_request = swap_request.into_inner();
    let quote = Jupiter::fetch_quote(
        &swap_request.input_mint.to_string(),
        &swap_request.output_mint.to_string(),
        swap_request.amount,
        swap_request.slippage,
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let keypair = state.wallet.lock().await.insecure_clone();
    let result = Jupiter::swap(quote, &keypair)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "result": result,
    })))
}
