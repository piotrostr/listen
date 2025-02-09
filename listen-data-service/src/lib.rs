#[ctor::ctor]
fn init() {
    #[cfg(test)]
    {
        std::env::set_var("RUST_LOG", "info");
        let _ = tracing_subscriber::fmt::try_init();
    }
}

pub mod constants;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "geyser")]
pub mod geyser;

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
