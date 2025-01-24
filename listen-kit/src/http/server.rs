use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;

use crate::http::routes::{stream, AppState};

pub async fn run_server(agent: Agent<CompletionModel>) -> std::io::Result<()> {
    let state = web::Data::new(AppState::new(agent));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(stream)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
