use crate::jup::Jupiter;
use crate::state::ServiceState;
use actix_web::{
    post,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SwapRequest {
    input_mint: String,
    output_mint: String,
    amount: u64,
    /// slippage in bps
    slippage: u16,
}

#[utoipa::path(
    post,
    path = "/swap",
    request_body = SwapRequest,
    responses(
        (status = 200, description = "Swap transaction successful"),
        (status = 400, description = "Invalid swap parameters"),
        (status = 500, description = "Swap transaction failed")
    ),
    tag = "swap"
)]
#[post("/swap")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_swap(
    swap_request: Json<SwapRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let swap_request = swap_request.into_inner();
    let quote = Jupiter::fetch_quote(
        &swap_request.input_mint,
        &swap_request.output_mint,
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
