#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "solana")]
pub mod solana;

#[cfg(feature = "evm")]
pub mod evm;

#[cfg(feature = "http")]
pub mod wallet_manager;

pub mod common;
pub mod cross_chain;
pub mod dexscreener;
pub mod reasoning_loop;
pub mod signer;

#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new("info")
                        .add_directive("listen_kit=info".parse().unwrap())
                }),
        )
        .with_test_writer()
        .try_init()
        .ok();
}
