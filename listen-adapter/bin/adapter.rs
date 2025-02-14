use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;

use listen_adapter::{
    db::make_db,
    redis_subscriber::create_redis_subscriber,
    routes::{get_candlesticks, health_check, top_tokens, ws_route},
    state::AppState,
    tls::load_rustls_config,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let redis_subscriber = create_redis_subscriber(&redis_url)
        .await
        .expect("Failed to create Redis subscriber");

    let clickhouse_db = make_db().expect("Failed to create Clickhouse DB");

    let app_state = AppState {
        redis_subscriber,
        clickhouse_db,
    };
    let app_data = web::Data::new(app_state);

    // Create the base app factory
    let app_factory = move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .route("/ws", web::get().to(ws_route))
            .route("/", web::get().to(health_check))
            .route("/top-tokens", web::get().to(top_tokens))
            .route("/candlesticks", web::get().to(get_candlesticks))
    };

    // Check if SSL certificates are configured
    let has_ssl = std::env::var("SSL_CERT_PATH").is_ok() && std::env::var("SSL_KEY_PATH").is_ok();

    if has_ssl {
        info!("SSL certificates found, starting HTTP/HTTPS servers");

        let http_server = HttpServer::new(app_factory.clone()).bind((host.clone(), 80))?;

        let rustls_config = load_rustls_config()?;
        let https_server =
            HttpServer::new(app_factory).bind_rustls_0_23((host.clone(), 443), rustls_config)?;

        info!("Starting WebSocket servers:");
        info!("ws://{}:80/ws", host);
        info!("wss://{}:443/ws", host);

        // Run both servers concurrently
        futures::future::try_join(http_server.run(), https_server.run()).await?;
    } else {
        // Start single server on port 6968
        info!("No SSL certificates found, starting regular WebSocket server");

        let port = 6968;
        HttpServer::new(app_factory)
            .bind((host.clone(), port))?
            .run()
            .await?;

        info!("WebSocket server running at ws://{}:{}/ws", host, port);
    }

    Ok(())
}
