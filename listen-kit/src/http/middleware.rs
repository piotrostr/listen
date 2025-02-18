use actix_web::{web, HttpRequest};
use anyhow::Result;
use privy::auth::UserSession;

use super::state::AppState;

pub async fn verify_auth(req: &HttpRequest) -> Result<UserSession> {
    println!("headers: {:#?}", req.headers());

    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let token = token.split(" ").nth(1).unwrap_or_default();

    println!("Extracted token: {}", token);

    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| anyhow::anyhow!("App state not found"))?;

    // Log before authentication attempt
    println!("Attempting to authenticate token...");

    match state.privy.authenticate_user(token).await {
        Ok(session) => {
            println!("Authentication successful!");
            Ok(session)
        }
        Err(e) => {
            println!("Authentication failed with error: {:?}", e);
            Err(anyhow::anyhow!("Invalid token: {}", e))
        }
    }
}
