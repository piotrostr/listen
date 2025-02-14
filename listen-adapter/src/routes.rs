use crate::websocket::handle_ws_connection;
use crate::{db::candlesticks::CandlestickInterval, state::AppState};
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;
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
        .get_top_tokens(20, None, None, Some(60 * 5), true)
        .await;

    match tokens {
        Ok(tokens) => Ok(HttpResponse::Ok().json(tokens)),
        Err(e) => {
            error!("Error getting top tokens: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}

#[derive(Deserialize)]
pub struct CandlestickParams {
    pub mint: String,
    pub interval: CandlestickInterval,
}

pub async fn get_candlesticks(
    state: web::Data<AppState>,
    query: web::Query<CandlestickParams>,
) -> Result<HttpResponse, Error> {
    let params = query.into_inner();
    let candlesticks = state
        .clickhouse_db
        .get_candlesticks(&params.mint, &params.interval.to_string())
        .await;

    match candlesticks {
        Ok(candlesticks) => Ok(HttpResponse::Ok().json(candlesticks)),
        Err(e) => {
            error!("Error getting candlesticks: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}

pub async fn get_metadata(
    state: web::Data<AppState>,
    query: web::Query<MetadataQuery>,
) -> Result<HttpResponse, Error> {
    match state.redis_client.get_metadata(&query.mint).await {
        Ok(Some(metadata)) => Ok(HttpResponse::Ok().json(metadata)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Metadata not found",
            "mint": query.mint
        }))),
        Err(e) => {
            error!("Error fetching metadata: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub sql: String,
}

pub async fn query_db(
    state: web::Data<AppState>,
    query: web::Json<QueryParams>,
) -> Result<HttpResponse, Error> {
    let result = state.clickhouse_db.generic_query(&query.sql).await;
    match result {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => {
            error!("Error querying database: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}

#[derive(Deserialize)]
pub struct MetadataQuery {
    mint: String,
}
