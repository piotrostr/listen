use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use listen_adapter::AdapterBuilder;
use listen_memory::graph::GraphMemory;
use privy::Privy;

use crate::http::routes::claims;

use super::routes::{auth, healthz, stream, suggest};
use super::state::AppState;
use listen_mongo::MongoClient;

pub async fn run_server(
    privy: Privy,
    mongo: Option<Arc<MongoClient>>,
    global_memory: Option<Arc<GraphMemory>>,
    with_adapter: bool,
    with_data_routes: bool,
    redis_url: Option<String>,
) -> std::io::Result<()> {
    let state = web::Data::new(
        AppState::new(privy, mongo, global_memory)
            .await
            .map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?,
    );

    // Initialize adapter if needed
    let adapter_state = if with_adapter {
        let redis_url =
            redis_url.expect("Redis URL required when adapter is enabled");
        let mut adapter_builder = AdapterBuilder::new(redis_url);
        if with_data_routes {
            adapter_builder = adapter_builder.with_data_routes();
        }
        Some(
            adapter_builder
                .with_websocket_routes()
                .with_route_prefix("")
                .build()
                .await
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    )
                })?,
        )
    } else {
        None
    };

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(state.clone())
            .service(healthz)
            .service(stream)
            .service(auth)
            .service(suggest)
            .service(claims);

        // Configure adapter routes if enabled
        if let Some(adapter_state) = adapter_state.as_ref() {
            app = app.configure(|config| {
                AdapterBuilder::configure_routes(
                    config,
                    adapter_state.clone(),
                    with_data_routes,
                    true,
                    true,
                    false,
                    "",
                );
            });
        }

        app
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
