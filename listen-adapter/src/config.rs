use actix_web::{web, Error};

use crate::{
    db::make_db,
    redis_client::make_redis_client,
    redis_subscriber::create_redis_subscriber,
    routes::{
        get_24h_open_price, get_candlesticks, get_chat, get_metadata, get_price, health_check,
        query_db, save_chat, top_tokens,
    },
    state::AppState,
};

pub async fn init_components(redis_url: &str, with_data_routes: bool) -> Result<AppState, Error> {
    let redis_client = make_redis_client()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let redis_subscriber = create_redis_subscriber(redis_url)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let clickhouse_db = if with_data_routes {
        Some(make_db().map_err(actix_web::error::ErrorInternalServerError)?)
    } else {
        None
    };

    Ok(AppState {
        redis_subscriber,
        redis_client,
        clickhouse_db,
    })
}

/// Creates and registers required components for data routes
///
/// # Arguments
/// * `config` - ServiceConfig to add routes to
/// * `state` - AppState with initialized components
pub fn configure_data_routes(config: &mut web::ServiceConfig, state: web::Data<AppState>) {
    // Register state
    config.app_data(state);

    // Add data routes
    config
        .route("/healthz", web::get().to(health_check))
        .route("/top-tokens", web::get().to(top_tokens))
        .route("/candlesticks", web::get().to(get_candlesticks))
        .route("/metadata", web::get().to(get_metadata))
        .route("/query", web::post().to(query_db))
        .route("/price", web::get().to(get_price))
        .route("/24h-open", web::get().to(get_24h_open_price));
}

/// Configures chat-related routes and components
pub fn configure_chat_routes(config: &mut web::ServiceConfig, state: web::Data<AppState>) {
    // Register state
    config.app_data(state);

    // Add chat routes
    config
        .route("/get-chat", web::get().to(get_chat))
        .route("/save-chat", web::post().to(save_chat));
}

/// Configures websocket routes and components
pub fn configure_websocket_routes(config: &mut web::ServiceConfig, state: web::Data<AppState>) {
    // Register state
    config.app_data(state);
}
