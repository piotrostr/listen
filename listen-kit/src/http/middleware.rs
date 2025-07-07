use actix_web::{web, HttpRequest};
use anyhow::Result;
use privy::{auth::UserSession, types::PrivyClaims};

use super::state::AppState;

pub async fn verify_auth(req: &HttpRequest) -> Result<UserSession> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;

    let token = token
        .strip_prefix("Bearer ")
        .ok_or_else(|| anyhow::anyhow!("Invalid authorization format"))?;

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| anyhow::anyhow!("App state not found"))?;

    match state.privy.authenticate_user(token).await {
        Ok(session) => Ok(session),
        Err(e) => Err(anyhow::anyhow!("Authentication failed: {}", e)),
    }
}

pub async fn verify_token(req: &HttpRequest) -> Result<PrivyClaims> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;

    let token = token
        .strip_prefix("Bearer ")
        .ok_or_else(|| anyhow::anyhow!("Invalid authorization format"))?;

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| anyhow::anyhow!("App state not found"))?;

    let claims = state.privy.validate_access_token(token)?;

    Ok(claims)
}
