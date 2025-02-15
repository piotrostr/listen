pub mod redis;
pub mod server;
pub mod trading_engine;

pub use trading_engine::TradingEngine;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }
}
