use crate::wallet_manager::UserSession;
use actix_web::{web, HttpRequest};
use anyhow::Result;

use super::state::AppState;

pub async fn verify_auth(req: &HttpRequest) -> Result<UserSession> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| anyhow::anyhow!("App state not found"))?;

    state
        .wallet_manager
        .authenticate_user(token)
        .await
        .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))
}
