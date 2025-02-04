pub mod trading_engine;

pub use trading_engine::TradingEngine;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
}
