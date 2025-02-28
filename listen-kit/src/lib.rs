use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "solana")]
pub mod solana;

#[cfg(feature = "evm")]
pub mod evm;

pub mod common;
pub mod cross_chain;
pub mod data;
pub mod dexscreener;
pub mod reasoning_loop;
pub mod signer;

#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();

    let is_systemd = std::env::var("IS_SYSTEMD_SERVICE").is_ok();

    // Configure logging based on environment
    if is_systemd {
        // Use systemd formatting when running as a service
        let journald_layer = tracing_journald::layer()
            .expect("Failed to create journald layer");
        tracing_subscriber::registry().with(journald_layer).init();
    } else {
        // Use standard formatting for non-systemd environments
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
}
