pub mod trading_engine;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
}
