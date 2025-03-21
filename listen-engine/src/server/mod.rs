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

pub mod cancel;
pub mod common;
pub mod create;
pub mod get;
pub mod internal;
pub mod state;

pub async fn run() -> std::io::Result<()> {
    let (server_tx, server_rx) = mpsc::channel(1000);
    tracing::info!("Created channel with capacity 1000");

    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }

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

    // Create a shared AppState for both servers
    let app_state = Data::new(AppState {
        engine_bridge_tx: server_tx.clone(),
        privy: privy.clone(),
    });

    // Create separate app states for each server
    let app_state_public = app_state.clone();
    let app_state_internal = app_state;

    // Main public server (unchanged)
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state_public.clone())
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
            .route(
                "/pipeline/{pipeline_id}/cancel",
                web::post().to(cancel::cancel_pipeline),
            )
            .route(
                "/pipeline/{pipeline_id}/step/{step_id}/cancel",
                web::post().to(cancel::cancel_step),
            )
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", 6966))?;

    // Create internal service-only server that only binds to localhost
    let internal_server = HttpServer::new(move || {
        App::new()
            .app_data(app_state_internal.clone())
            .wrap(middleware::Logger::default())
            .route(
                "/internal/create_pipeline",
                web::post().to(internal::create_pipeline_internal),
            )
    })
    .bind(("127.0.0.1", 6901))?; // Different port, localhost only

    // Run both servers
    let server_future = server.run();
    let internal_server_future = internal_server.run();

    tokio::select! {
        result = server_future => {
            engine.shutdown().await;
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        result = internal_server_future => {
            engine.shutdown().await;
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Internal server error: {}", e);
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
