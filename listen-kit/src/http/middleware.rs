use actix_web::{web, HttpRequest};
use anyhow::Result;
use privy::auth::UserSession;

use super::state::AppState;

pub async fn verify_auth(req: &HttpRequest) -> Result<UserSession> {
    tracing::info!("headers: {:#?}", req.headers());

    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;

    let token = token
        .strip_prefix("Bearer ")
        .ok_or_else(|| anyhow::anyhow!("Invalid authorization format"))?;

    tracing::debug!("Extracted token: {}", token);

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| anyhow::anyhow!("App state not found"))?;

    // Log Privy configuration
    tracing::info!(
        "Privy Configuration - App ID: {}, Public Key Length: {}",
        state.privy.config.app_id,
        state.privy.config.verification_key.len()
    );

    tracing::info!("token is: {}", token);

    // Add more detailed logging
    tracing::info!("Attempting to authenticate token...");

    match state.privy.authenticate_user(token).await {
        Ok(session) => {
            tracing::info!(
                "Authentication successful for user: {}",
                session.user_id
            );
            Ok(session)
        }
        Err(e) => {
            // Log the detailed error and configuration context
            tracing::error!(
                "Authentication failed: {:?}\nApp ID: {}\nPublic Key Length: {}", 
                e,
                state.privy.config.app_id,
                state.privy.config.verification_key.len()
            );
            tracing::error!("{}", state.privy.config.verification_key);

            Err(anyhow::anyhow!("Authentication failed: {}", e))
        }
    }
}
