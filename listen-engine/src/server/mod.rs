use actix_cors::Cors;
use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::{engine::Engine, metrics::metrics_handler, server::state::AppState};
use privy::{config::PrivyConfig, Privy};

pub mod create;
pub mod get;
pub mod state;

pub async fn run() -> std::io::Result<()> {
    let (server_tx, server_rx) = mpsc::channel(1000);
    tracing::info!("Created channel with capacity 1000");

    // Create engine and get price update receiver
    let (engine, price_rx) = match Engine::from_env().await {
        Ok((engine, rx)) => (engine, rx),
        Err(e) => {
            tracing::error!("Failed to create engine: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create engine",
            ));
        }
    };
    let engine = Arc::new(engine);

    // Create a shutdown signal handler
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    // Set up ctrl-c handler
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            let _ = shutdown_tx_clone.send(()).await;
        }
    });

    let privy = Arc::new(Privy::new(PrivyConfig::from_env().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to create privy config")
    })?));

    // Main application server with metrics endpoint
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                engine_bridge_tx: server_tx.clone(),
                privy: privy.clone(),
            }))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .expose_headers(["content-type", "authorization"])
                    .max_age(3600)
                    .allowed_header(actix_web::http::header::CONTENT_TYPE)
                    .allowed_header(actix_web::http::header::AUTHORIZATION),
            )
            .wrap(middleware::Logger::default())
            .route("/healthz", web::get().to(healthz))
            .route("/pipeline", web::post().to(create::create_pipeline))
            .route("/pipelines", web::get().to(get::get_pipelines))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", 6966))?
    .run();

    tokio::select! {
        result = server => {
            engine.shutdown().await;
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        result = Engine::run(engine.clone(), price_rx, server_rx) => {
            engine.shutdown().await;
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Engine error: {}", e);
            }
        }
        _ = shutdown_rx.recv() => {
            tracing::info!("Shutdown signal received, starting graceful shutdown");
            engine.shutdown().await;
        }
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn healthz() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}
