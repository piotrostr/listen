use crate::websocket::handle_ws_connection;
use crate::{db::candlesticks::CandlestickInterval, state::AppState};
use actix_web::{error::InternalError, http::StatusCode, web, Error, HttpRequest, HttpResponse};
use regex::Regex;
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

pub async fn version() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "version": "2.5.0"
    }))
}

#[derive(Deserialize)]
pub struct TopTokensQuery {
    pub limit: Option<usize>,
    pub min_volume: Option<f64>,
    pub min_market_cap: Option<f64>,
    pub max_market_cap: Option<f64>,
    pub timeframe: Option<u64>,
    pub only_pumpfun_tokens: Option<bool>,
}

pub async fn top_tokens(
    state: web::Data<AppState>,
    query: web::Query<TopTokensQuery>,
) -> Result<HttpResponse, Error> {
    let tokens = state
        .clickhouse_db
        .get_top_tokens(
            query.limit.unwrap_or(20),
            query.min_volume,
            query.min_market_cap,
            query.max_market_cap,
            query.timeframe,
            query.only_pumpfun_tokens.unwrap_or(true),
        )
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
    pub limit: Option<usize>,
}

pub async fn get_candlesticks(
    state: web::Data<AppState>,
    query: web::Query<CandlestickParams>,
) -> Result<HttpResponse, Error> {
    let params = query.into_inner();
    let candlesticks = state
        .clickhouse_db
        .get_candlesticks(&params.mint, &params.interval.to_string(), params.limit)
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
pub struct PriceQuery {
    pub mint: String,
}

pub async fn get_price(
    state: web::Data<AppState>,
    query: web::Query<PriceQuery>,
) -> Result<HttpResponse, Error> {
    let price = state.redis_client.get_price(&query.mint).await;
    match price {
        Ok(price) => Ok(HttpResponse::Ok().json(price)),
        Err(e) => {
            error!("Error getting price: {}", e);
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
    // Sanitize the SQL query
    let sql = query.sql.trim();

    // Only allow SELECT queries (case insensitive check)
    if !sql.to_uppercase().starts_with("SELECT ") {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Only SELECT queries are allowed"
        })));
    }

    // Block dangerous SQL commands while allowing table names containing these words
    const BLOCKED_PATTERNS: [&str; 7] = [
        r"(?i)\bDELETE\b",
        r"(?i)\bDROP\b",
        r"(?i)\bUPDATE\b",
        r"(?i)\bINSERT\b",
        r"(?i)\bALTER\b",
        r"(?i)\bTRUNCATE\b",
        r"(?i)\bGRANT\b",
    ];

    for pattern in BLOCKED_PATTERNS {
        if Regex::new(pattern).unwrap().is_match(sql) {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Query contains forbidden SQL commands"
            })));
        }
    }

    // Execute the validated query
    let result = state.clickhouse_db.generic_query(sql).await;
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

#[derive(Deserialize)]
pub struct ChatQuery {
    pub chat_id: String,
}

pub async fn get_chat(
    state: web::Data<AppState>,
    query: web::Query<ChatQuery>,
) -> Result<HttpResponse, Error> {
    let chat = state.redis_client.get_chat(&query.chat_id).await;
    match chat {
        Ok(Some(chat)) => Ok(HttpResponse::Ok().json(chat)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Chat not found",
            "chat_id": query.chat_id
        }))),
        Err(e) => {
            error!("Error getting chat: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}

#[derive(Deserialize)]
pub struct SaveChatRequest {
    pub chat_id: String,
    pub chat: serde_json::Value,
}

pub async fn save_chat(
    state: web::Data<AppState>,
    body: web::Json<SaveChatRequest>,
) -> Result<HttpResponse, Error> {
    let chat = state
        .redis_client
        .save_chat(&body.chat_id, &body.chat)
        .await;
    match chat {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "message": "Chat saved",
            "chat_id": body.chat_id
        }))),
        Err(e) => {
            error!("Error saving chat: {}", e);
            Err(InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
        }
    }
}
