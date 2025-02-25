use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use privy::Privy;

use super::routes::{auth, healthz, stream};
use super::state::AppState;

pub async fn run_server(privy: Privy) -> std::io::Result<()> {
    let state = web::Data::new(AppState::new(privy));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(healthz)
            .service(stream)
            .service(auth)
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
