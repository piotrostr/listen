pub mod constants;
pub mod kv_store;
pub mod message_queue;
pub mod metadata;
pub mod pipeline;
pub mod price;
pub mod raydium_processor;
pub mod util;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
}
