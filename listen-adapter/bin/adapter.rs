use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;

use listen_adapter::{
    db::make_db,
    redis_client::make_redis_client,
    redis_subscriber::create_redis_subscriber,
    routes::{get_candlesticks, get_metadata, health_check, query_db, top_tokens, ws_route},
    state::AppState,
};

#[cfg(feature = "tls")]
use listen_adapter::tls::load_rustls_config;

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

    let redis_client = make_redis_client()
        .await
        .expect("Failed to create Redis client");

    let app_state = AppState {
        redis_subscriber,
        redis_client,
        clickhouse_db,
    };
    let app_data = web::Data::new(app_state);

    // Create the base app factory
    let app_factory = move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::default().allow_any_origin())
            .app_data(app_data.clone())
            .service(
                web::scope("/v1/adapter")
                    .route("/ws", web::get().to(ws_route))
                    .route("/healthz", web::get().to(health_check))
                    .route("/top-tokens", web::get().to(top_tokens))
                    .route("/candlesticks", web::get().to(get_candlesticks))
                    .route("/metadata", web::get().to(get_metadata))
                    .route("/query", web::post().to(query_db)),
            )
    };

    #[cfg(feature = "tls")]
    let has_ssl = std::env::var("SSL_CERT_PATH").is_ok() && std::env::var("SSL_KEY_PATH").is_ok();
    #[cfg(not(feature = "tls"))]
    let has_ssl = false;

    if has_ssl {
        #[cfg(feature = "tls")]
        {
            info!("SSL certificates found, starting HTTP/HTTPS servers");

            let http_server = HttpServer::new(app_factory.clone()).bind((host.clone(), 80))?;

            let rustls_config = load_rustls_config()?;
            let https_server = HttpServer::new(app_factory)
                .bind_rustls_0_23((host.clone(), 443), rustls_config)?;

            info!("Starting WebSocket servers:");
            info!("ws://{}:80/v1/adapter/ws", host);
            info!("wss://{}:443/v1/adapter/ws", host);

            futures::future::try_join(http_server.run(), https_server.run()).await?;
        }

        #[cfg(not(feature = "tls"))]
        {
            unreachable!("TLS support is not enabled");
        }
    } else {
        let port = 6968;
        info!("Starting WebSocket server on port {}", port);

        HttpServer::new(app_factory)
            .bind((host.clone(), port))?
            .run()
            .await?;

        info!(
            "WebSocket server running at ws://{}:{}/v1/adapter/ws",
            host, port
        );
    }

    Ok(())
}
