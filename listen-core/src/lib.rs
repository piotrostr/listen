/// # listen
///
/// Blazingly fast actions for AI Agents with a simple API.
///
/// ## Quick Start
///
/// ```rust
/// use listen::{actions::Actions, util::env, constants::WSOL};
/// use solana_sdk::native_token::sol_to_lamports;
///
/// #[tokio::main]
/// async fn main() {
///    dotenv::dotenv().ok();
///    let actions = Actions::new(env("PRIVATE_KEY"), env("RPC_URL"));
///
///    let balance = actions.get_balance().await.unwrap();
///    println!("Balance: {}", balance);
///
///    let mint = "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump".to_string();
///    let price = actions.fetch_token_price(mint.clone()).await.unwrap();
///    println!("Price: {}", price);
///
///    // Trade 0.01 SOL for $Fartcoin
///    let slippage_bps = 100;
///    let tx_id = actions.trade(
///        WSOL.to_string(),
///        sol_to_lamports(0.01),
///        mint,
///        slippage_bps
///    ).await.unwrap();
///    println!("Signature: {}", tx_id);
/// }
/// ```
pub mod actions;
pub mod agent;
pub mod balance;
pub mod constants;
pub mod deploy_token;
pub mod jito;
pub mod jup;
pub mod price;
pub mod pump;
pub mod trade;
pub mod transfer;
pub mod util;
