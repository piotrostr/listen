use crate::http::{middleware::verify_auth, state::AppState};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use anyhow::Result;
use serde_json::json;

#[get("/auth")]
async fn auth(req: HttpRequest) -> Result<HttpResponse, Error> {
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
    })))
}
