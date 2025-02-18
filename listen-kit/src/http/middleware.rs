use actix_web::{web, HttpRequest};
use anyhow::Result;
use privy::auth::UserSession;

use super::state::AppState;

pub async fn verify_auth(req: &HttpRequest) -> Result<UserSession> {
    println!("headers: {:#?}", req.headers());

    // Log the full authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    println!("Authorization header: {:?}", auth_header);

    let token = auth_header
        .and_then(|s| s.split(" ").nth(1))
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?
        .trim();

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
