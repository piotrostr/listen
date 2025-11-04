use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use listen_tracing::setup_tracing;
use tracing::info;

use listen_adapter::AdapterBuilder;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    setup_tracing();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let with_data_routes =
        std::env::var("WITH_DATA_ROUTES").unwrap_or_else(|_| "false".to_string()) == "true";

    // Initialize components once
    let builder = AdapterBuilder::new(redis_url);
    let builder = if with_data_routes {
        builder.with_data_routes()
    } else {
        builder
    };
    let builder = builder
        .with_chat_routes()
        .with_websocket_routes()
        .with_health_check()
        .with_route_prefix(""); // Empty prefix since this is the main adapter

    let app_state = builder.build().await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to initialize components: {}", e),
        )
    })?;

    let app_factory = move || {
        let app = App::new().wrap(Logger::default()).wrap(Cors::permissive());

        // Configure routes with shared state
        let app_state = app_state.clone();
        app.configure(|config| {
            AdapterBuilder::configure_routes(
                config,
                app_state.clone(),
                with_data_routes,
                true, // chat routes
                true, // websocket routes
                true, // health check
                "",   // empty prefix
            );
        })
    };

    let port = 6968;
    info!("Starting WebSocket server on port {}", port);

    HttpServer::new(app_factory)
        .bind((host.as_str(), port))?
        .run()
        .await?;

    info!("WebSocket server running at ws://{}:{}", host, port);

    Ok(())
}
