pub mod engine;
pub mod redis;
pub mod server;

pub use engine::Engine;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }
}
