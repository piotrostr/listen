use crate::state::AppState;
use crate::websocket::handle_ws_connection;
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use serde_json::json;
use tracing::error;

pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn WebSocket handler
    actix_web::rt::spawn(handle_ws_connection(
        session,
        msg_stream,
        state.redis_subscriber.clone(),
    ));

    Ok(res)
}

pub async fn health_check() -> HttpResponse {
    let timestamp = chrono::Utc::now().timestamp();
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "timestamp": timestamp
    }))
}

pub async fn top_tokens(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let tokens = state
        .clickhouse_db
        .get_top_tokens(20, None, None, Some(60 * 5))
        .await;

    match tokens {
        Ok(tokens) => Ok(HttpResponse::Ok().json(tokens)),
        Err(e) => {
            error!("Error getting top tokens: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}
