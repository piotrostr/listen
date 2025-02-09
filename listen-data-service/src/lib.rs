pub mod account_pipeline;
pub mod constants;
pub mod instruction_pipeline;
pub mod jupiter_processor;
pub mod kv_store;
pub mod message_queue;
pub mod metadata;
pub mod price;
pub mod raydium_intruction_processor;
pub mod raydium_processor;
pub mod sol_price_stream;
pub mod util;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
}
