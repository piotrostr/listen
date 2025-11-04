pub mod config;
pub mod db;
pub mod error;
pub mod redis_client;
pub mod redis_subscriber;
pub mod routes;
pub mod state;
pub mod version;
pub mod webhook;
pub mod websocket;

use actix_web::{web, Error};
use state::AppState;

pub struct AdapterBuilder {
    redis_url: String,
    with_data_routes: bool,
    with_chat_routes: bool,
    with_websocket_routes: bool,
    with_health_check: bool,
    route_prefix: String,
}

impl AdapterBuilder {
    pub fn new(redis_url: impl Into<String>) -> Self {
        Self {
            redis_url: redis_url.into(),
            with_data_routes: false,
            with_chat_routes: false,
            with_websocket_routes: false,
            with_health_check: false,
            route_prefix: "/adapter".to_string(),
        }
    }

    pub fn with_data_routes(mut self) -> Self {
        self.with_data_routes = true;
        self
    }

    pub fn with_chat_routes(mut self) -> Self {
        self.with_chat_routes = true;
        self
    }

    pub fn with_websocket_routes(mut self) -> Self {
        self.with_websocket_routes = true;
        self
    }

    pub fn with_health_check(mut self) -> Self {
        self.with_health_check = true;
        self
    }

    pub fn with_route_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.route_prefix = prefix.into();
        self
    }

    pub async fn build(self) -> Result<web::Data<AppState>, Error> {
        let app_state = config::init_components(&self.redis_url, self.with_data_routes).await?;
        Ok(web::Data::new(app_state))
    }

    pub fn configure_routes(
        config: &mut web::ServiceConfig,
        state: web::Data<AppState>,
        with_data: bool,
        with_chat: bool,
        with_websocket: bool,
        with_health_check: bool,
        route_prefix: &str,
    ) {
        let mut scope = web::scope(route_prefix).app_data(state.clone());

        if with_health_check {
            scope = scope.route("/healthz", web::get().to(routes::health_check));
        }

        scope = scope
            .route("/webhook", web::post().to(webhook::webhook))
            .route("/ws", web::get().to(routes::ws_route));

        if with_data {
            scope = scope.configure(|s| config::configure_data_routes(s, state.clone()));
        }
        if with_chat {
            scope = scope.configure(|s| config::configure_chat_routes(s, state.clone()));
        }
        if with_websocket {
            scope = scope.configure(|s| config::configure_websocket_routes(s, state.clone()));
        }

        config.service(scope);
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    listen_tracing::setup_tracing();
}
