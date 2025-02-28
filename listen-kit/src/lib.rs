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
    listen_tracing::setup_tracing();
}
