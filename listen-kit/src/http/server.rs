use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use listen_memory::graph::GraphMemory;
use privy::Privy;

use super::routes::{auth, healthz, stream, suggest};
use super::state::AppState;
use listen_mongo::MongoClient;

pub async fn run_server(
    privy: Privy,
    mongo: Option<Arc<MongoClient>>,
    global_memory: Option<Arc<GraphMemory>>,
) -> std::io::Result<()> {
    let state = web::Data::new(
        AppState::new(privy, mongo, global_memory)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
    );

    HttpServer::new(move || {
        App::new()
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
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
