use crate::http::{
    middleware::{verify_auth, verify_token},
    state::AppState,
};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use anyhow::Result;
use serde_json::json;

#[get("/claims")]
async fn claims(req: HttpRequest) -> Result<HttpResponse, Error> {
    let privy_claims = match verify_token(&req).await {
        Ok(privy_claims) => privy_claims,
        Err(e) => {
            return Ok(HttpResponse::Unauthorized()
                .json(json!({ "error": e.to_string() })))
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "claims": privy_claims,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}

#[get("/auth")]
async fn auth(req: HttpRequest) -> Result<HttpResponse, Error> {
    let privy_claims = match verify_token(&req).await {
        Ok(privy_claims) => privy_claims,
        Err(e) => {
            return Ok(HttpResponse::Unauthorized()
                .json(json!({ "error": e.to_string() })))
        }
    };

    let user_session = match verify_auth(&req).await {
        Ok(session) => session,
        Err(e) => {
            return Ok(HttpResponse::Unauthorized()
                .json(json!({ "error": e.to_string() })))
        }
    };

    let state = match req.app_data::<web::Data<AppState>>() {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::InternalServerError()
                .json(json!({ "error": "App state not found" })))
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "wallet_address": user_session.wallet_address,
        "user_id": user_session.user_id,
        "privy_app_id": state.privy.config.app_id,
        "claims": privy_claims,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}
