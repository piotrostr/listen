use crate::{engine::error::EngineError, server::state::AppState};
use actix_web::{web::Data, HttpRequest, HttpResponse};
use privy::auth::UserSession;
use serde::Serialize;
use tokio::sync::oneshot;

pub async fn verify_auth(
    state: &Data<AppState>,
    req: &HttpRequest,
) -> Result<UserSession, HttpResponse> {
    // Extract and validate auth token
    let auth_token = match req.headers().get("authorization") {
        Some(auth_token) => auth_token.to_str().unwrap_or(""),
        None => {
            return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Authorization header is required"
            })));
        }
    };

    let auth_token = match auth_token.split(" ").nth(1) {
        Some(token) => token,
        None => {
            return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Invalid authorization format"
            })));
        }
    };

    // Authenticate the user
    match state
        .privy
        .authenticate_user(auth_token)
        .await
        .map_err(|_| HttpResponse::Unauthorized())
    {
        Ok(user) => Ok(user),
        Err(_) => Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "Unauthorized"
        }))),
    }
}

pub async fn handle_engine_response<T: Serialize>(
    response_rx: oneshot::Receiver<Result<T, EngineError>>,
    success_message: &str,
) -> HttpResponse {
    match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
        Ok(response) => match response {
            Ok(Ok(response)) => HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": success_message,
                "response": response
            })),
            Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Operation failed: {}", e)
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to receive response from engine: {}", e)
            })),
        },
        Err(_) => HttpResponse::GatewayTimeout().json(serde_json::json!({
            "status": "error",
            "message": "Operation timed out"
        })),
    }
}
