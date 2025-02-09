pub mod account_pipeline;
pub mod constants;
pub mod instruction_pipeline;
pub mod jupiter_processor;
pub mod kv_store;
pub mod message_queue;
pub mod metadata;
pub mod price;
pub mod process_swap;
pub mod raydium_intruction_processor;
pub mod raydium_processor;
pub mod sol_price_stream;
pub mod util;

#[cfg(test)]
pub mod debug;

#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();

    #[cfg(test)]
    {
        std::env::set_var("RUST_LOG", "info");
        let _ = tracing_subscriber::fmt::try_init();
    }
}
