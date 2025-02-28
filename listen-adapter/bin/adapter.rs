use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use listen_adapter::{
    db::make_db,
    redis_client::make_redis_client,
    redis_subscriber::create_redis_subscriber,
    routes::{
        get_candlesticks, get_metadata, get_price, health_check, query_db, top_tokens, ws_route,
    },
    state::AppState,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let is_systemd = std::env::var("IS_SYSTEMD_SERVICE").is_ok();

    // Configure logging based on environment
    if is_systemd {
        // Use systemd formatting when running as a service
        let journald_layer = tracing_journald::layer().expect("Failed to create journald layer");
        tracing_subscriber::registry().with(journald_layer).init();
    } else {
        // Use standard formatting for non-systemd environments
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_target(true)
            .init();
    }

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let redis_subscriber = create_redis_subscriber(&redis_url)
        .await
        .expect("Failed to create Redis subscriber");

    let clickhouse_db = make_db().expect("Failed to create Clickhouse DB");

    let redis_client = make_redis_client()
        .await
        .expect("Failed to create Redis client");

    let app_state = AppState {
        redis_subscriber,
        redis_client,
        clickhouse_db,
    };
    let app_data = web::Data::new(app_state);

    let app_factory = move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::default().allow_any_origin())
            .app_data(app_data.clone())
            .route("/ws", web::get().to(ws_route))
            .route("/healthz", web::get().to(health_check))
            .route("/top-tokens", web::get().to(top_tokens))
            .route("/candlesticks", web::get().to(get_candlesticks))
            .route("/metadata", web::get().to(get_metadata))
            .route("/query", web::post().to(query_db))
            .route("/price", web::get().to(get_price))
    };

    let port = 6968;
    info!("Starting WebSocket server on port {}", port);

    HttpServer::new(app_factory)
        .bind((host.clone(), port))?
        .run()
        .await?;

    info!("WebSocket server running at ws://{}:{}", host, port);

    Ok(())
}
