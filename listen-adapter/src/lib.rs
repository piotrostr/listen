pub mod db;
pub mod error;
pub mod redis_client;
pub mod redis_subscriber;
pub mod routes;
pub mod state;
pub mod websocket;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    listen_tracing::setup_tracing();
}
