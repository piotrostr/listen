#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "solana")]
pub mod solana;

#[cfg(feature = "evm")]
pub mod evm;

pub mod agent;

pub mod wallet_manager;

#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();
}
