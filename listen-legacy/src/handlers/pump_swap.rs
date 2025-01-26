use std::str::FromStr;

use crate::jito::send_jito_tx;
use crate::pump::{
    _make_buy_ixs, get_bonding_curve, get_token_amount, make_pump_sell_ix,
    mint_to_pump_accounts,
};
use crate::state::ServiceState;
use actix_web::{
    post,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PumpBuyRequest {
    mint: String,

    /// sol_amount denoted in lamports
    sol_amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PumpSellRequest {
    mint: String,

    /// token_amount to sell
    token_amount: u64,

    /// slippage in bps
    slippage: u16,
}

#[utoipa::path(
    post, 
    path = "/pump-buy",
    request_body = PumpBuyRequest,
    responses((status = 200)),
    tag = "pump-swap"
)]
#[post("/pump-buy")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pump_buy(
    pump_buy_request: Json<PumpBuyRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let pump_buy_request = pump_buy_request.into_inner();
    let mint = Pubkey::from_str(&pump_buy_request.mint)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let pump_accounts = mint_to_pump_accounts(&mint)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let bonding_curve =
        get_bonding_curve(&state.rpc_client, pump_accounts.bonding_curve)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    let token_amount = get_token_amount(
        bonding_curve.virtual_sol_reserves,
        bonding_curve.virtual_token_reserves,
        bonding_curve.real_token_reserves,
        pump_buy_request.sol_amount,
    )?;

    let keypair = state.wallet.lock().await.insecure_clone();

    let owner = keypair.pubkey();

    let buy_ixs = _make_buy_ixs(
        owner,
        pump_accounts.mint,
        pump_accounts.bonding_curve,
        pump_accounts.associated_bonding_curve,
        token_amount,
        pump_buy_request.sol_amount,
    )
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let latest_blockhash = *state.latest_blockhash.lock().await;

    let tx = Transaction::new_signed_with_payer(
        buy_ixs.as_slice(),
        Some(&owner),
        &[&keypair],
        latest_blockhash,
    );

    let result = send_jito_tx(tx)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "result": result,
    })))
}

#[utoipa::path(
    post, 
    path = "/pump-sell", 
    request_body = PumpSellRequest, 
    responses((status = 200, description = "Pump sell transaction successful")),
    tag = "pump-swap"
)]
#[post("/pump-sell")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pump_sell(
    pump_sell_request: Json<PumpSellRequest>,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let pump_sell_request = pump_sell_request.into_inner();
    let mint = Pubkey::from_str(&pump_sell_request.mint)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let pump_accounts = mint_to_pump_accounts(&mint)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let keypair = state.wallet.lock().await.insecure_clone();

    let owner = keypair.pubkey();

    let ata = spl_associated_token_account::get_associated_token_address(
        &owner,
        &pump_accounts.mint,
    );

    let ix = make_pump_sell_ix(
        owner,
        pump_accounts,
        pump_sell_request.token_amount,
        ata,
    )
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let latest_blockhash = *state.latest_blockhash.lock().await;

    let tx = Transaction::new_signed_with_payer(
        [ix].as_slice(),
        Some(&owner),
        &[&keypair],
        latest_blockhash,
    );

    let result = send_jito_tx(tx)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "result": result,
    })))
}
