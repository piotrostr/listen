//! # Listen
//!
//! Solana Swiss-Knife
//!
//! Supports all blockchain interactions required for algorithmic meme coin trading in Rust, with
//! a dedicated `snipe` module for insta-buys and AI Agent integration
//!
//! Pump.fun specific version
//!
//! ## Installation
//!
//! ```sh
//! sudo apt install protoc build-essential pkg-config libssl-dev
//! git clone https://github.com/piotrostr/listen && cd listen
//! cp .env.example .env  # swap the example values with your RPCs
//! cargo build --release
//! ```
//!
//! ## Example (top holders check)
//!
//! The example uses the `check_top_holders` function from the `listen` lib to check the top
//! holders in combination with the `rig` framework from [arc](https://arc.fun) to create a simple
//! AI Agent callable from CLI.
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
//!             let rpc_client = RpcClient::new(env("RPC_URL"));
//!             let result = check_top_holders(&mint, &rpc_client, true).await;
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
//! Then, in your `main.rs` file, you can use the `rig` framework to run the agent:
//!
//! ```rust
//! #[tokio::main]
//! pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//!     let client = openai::Client::from_env();
//!
//!     let agent = client
//!         .agent(openai::GPT_4O)
//!         .preamble("I can help you check token holder concentration.")
//!         .max_tokens(2048)
//!         .tool(TopHolders)
//!         .build();
//!
//!     let holders_response = agent
//!         .prompt(
//!             "Check top holders for mint address GJAFwWjJ3vnTsrQVabjBVK2TYB1YtRCQXRDfDgUnpump, using the `top_holders` tool",
//!         )
//!         .await?;
//!
//!     // example of using of the output
//!     let analysis = agent
//!         .prompt(&format!(
//!             "{}: {}",
//!             "is this a safe top 10 holders distribution?", holders_response
//!         ))
//!         .await?;
//!
//!     println!("{}", analysis);
//!
//!     Ok(())
//! }
//! ```
//!
//! Full Code: [src/agent.rs](https://github.com/piotrostr/listen/blob/main/src/agent.rs)
//!
//! You can run this example with `cargo run --release arc-agent`.
//!
//! ## Environment Variables
//!
//! - First section are the Jito parameters, which provide the fastest transaction execution and
//!   submitting transaction bundles
//!
//! - The `AUTH_KEYPAIR_PATH` is a path to `solana-keygen` generated keypair, which has to be
//!   pre-approved by Jito for using the gRPC HTTP/2.0 client with best latency
//!
//! - The `FUND_KEYPAIR_PATH` is the wallet path, to be used as a "fund wallet" that executes
//!   transactions
//!
//! - The last section is only required for running the library `snipe` module, which spawns
//!   4 micro-services responsible for listening on new listings, pipeline of subscribe for new
//!   listings, send to checker for verification, if checks are OK, send to buyer for purchase;
//!   lastly seller service manages the sl/tp; The sniper is super fast, executes transactions in
//!   1-5 blocks from token creation; Example of a wallet managed by this algorithm can be found
//!   here: [FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi](https://solscan.io/address/FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi)
//!
//! ```txt
//! BLOCK_ENGINE_URL=https://frankfurt.mainnet.block-engine.jito.wtf
//! SHRED_RECEIVER_ADDR=145.40.93.84:1002
//! RELAYER_URL=http://frankfurt.mainnet.relayer.jito.wtf:8100
//!
//! AUTH_KEYPAIR_PATH=auth.json
//! FUND_KEYPAIR_PATH=fund.json
//!
//! WS_URL=wss://api.mainnet-beta.solana.com
//! RPC_URL=https://api.mainnet-beta.solana.com
//!
//! MONGO_URL="mongodb+srv://<username>:<password>@cluster0.ifvf463.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0"
//!
//! LISTENER_URL=http://localhost:8078
//! CHECKER_URL=http://localhost:8079
//! BUYER_URL=http://localhost:8080
//! SELLER_URL=http:/localhost:8081
//!
//! OPENAI_API_KEY=sk-<your-openai-api-key>
//! ```
//!
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
pub mod api_docs;
pub mod app;
pub mod ata;
pub mod blockhash;
pub mod buyer;
pub mod buyer_service;
pub mod checker;
pub mod checker_service;
pub mod collector;
pub mod constants;
pub mod execute;
pub mod handlers;
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
pub mod service;
pub mod state;
pub mod tx_parser;
pub mod types;
pub mod util;
pub mod ws;

mod tests;

pub use crate::listener::*;
pub use crate::provider::*;
