//! # Listen
//!
//! Set of tools for all blockchain interactions for algorithmic meme coin trading in Rust with
//! support for AI agents
//!
//! ## Installation
//!
//! ```bash
//! sudo apt install protoc build-essential pkg-config libssl-dev
//! git clone https://github.com/piotrostr/listen && cd listen
//! cp .env.example .env  # swap the example values with your RPCs
//! cargo build --release
//! ```
//!
//! ## Example (top holders check)
//!
//! The example uses the `check_top_holders` function from the `listen` lib to check the top
//! holders in combination with the `rig` framework from [arc](arc.fun) to create a simple CLI AI
//! Agent
//!
//! ```rust
//! // import the check_top_holders from \`listen\` lib
//! use listen::buyer::check_top_holders;
//! // import the rig framework Tool trait
//! use rig::{completion::ToolDefinition, tool::Tool};
//!
//! #[derive(Deserialize, Serialize)]
//! pub struct TopHolders;
//! impl Tool for TopHolders {
//!     async fn call(
//!         &self,
//!         args: Self::Args,
//!     ) -> Result<Self::Output, Self::Error> {
//!         let mint = Pubkey::from_str(&args.mint)
//!             .map_err(|e| TopHoldersError::InvalidMint(e.to_string()))?;
//!
//!         // Create a channel
//!         let (tx, mut rx) = mpsc::channel(1);
//!
//!         // Spawn a task to handle the RPC calls
//!         tokio::spawn(async move {
//!             let provider = Provider::new(env("RPC_URL"));
//!             let result = check_top_holders(&mint, &provider, true).await;
//!             let _ = tx.send(result).await;
//!         });
//!
//!         // Wait for the result
//!         let result = rx
//!             .recv()
//!             .await
//!             .ok_or_else(|| {
//!               TopHoldersError::CheckFailed("Channel closed".to_string())
//!             })?
//!             .map_err(|e| TopHoldersError::CheckFailed(e.to_string()))?;
//!
//!         let (percentage, is_concentrated, details) = result;
//!
//!         Ok(TopHoldersOutput {
//!             percentage,
//!             is_concentrated,
//!             details,
//!         })
//!     }
//! }
//! ```
//!
//! Full Code: [src/agent.rs](https://github.com/piotrostr/listen/blob/main/src/agent.rs)
//!
//! ## All Actions
//!
//! ```txt
//! $ listen
//! Usage: listen [OPTIONS] <COMMAND>
//!
//! Commands:
//!   close-token-accounts
//!   pump-service
//!   grab-metadata
//!   sell-pump
//!   bump-pump
//!   sweep-pump
//!   snipe-pump
//!   buy-pump-token
//!   generate-custom-address
//!   ata
//!   spl-stream
//!   monitor-mempool
//!   seller-service
//!   checker-service
//!   checks
//!   blockhash
//!   listen-for-sol-pooled
//!   buyer-service
//!   track-position
//!   top-holders
//!   monitor-leaders
//!   monitor-slots
//!   price
//!   bench-rpc
//!   priority-fee
//!   tx
//!   listen
//!   listen-for-burn
//!   listener-service
//!   snipe
//!   wallet
//!   parse-pool
//!   swap
//!   help                 Print this message or the help of the given subcommand(s)
//!
//! Options:
//!   -u, --url <URL>                 [default: https://api.mainnet-beta.solana.com]
//!   -w, --ws-url <WS_URL>           [default: wss://api.mainnet-beta.solana.com]
//!   -k, --keypair-path <KEYPAIR_PATH>
//!       --tokio-console
//!   -h, --help                      Print help
//!   -V, --version                   Print version
//! ```

pub mod address;
pub mod agent;
pub mod app;
pub mod ata;
pub mod buyer;
pub mod buyer_service;
pub mod checker;
pub mod checker_service;
pub mod collector;
pub mod constants;
pub mod execute;
pub mod http_client;
pub mod jito;
pub mod jup;
pub mod listener;
pub mod listener_service;
pub mod orca;
pub mod prometheus;
pub mod provider;
pub mod pump;
pub mod pump_service;
pub mod raydium;
pub mod rpc;
pub mod seller;
pub mod seller_service;
pub mod tx_parser;
pub mod types;
pub mod util;

mod tests;

pub use crate::listener::*;
pub use crate::provider::*;
