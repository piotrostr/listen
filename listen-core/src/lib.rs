/// # listen
///
/// Blazingly fast actions for AI Agents with a simple API.
///
/// ## Example
///
/// ```rust
/// use listen::{actions::Actions, constants::WSOL, util::sol_to_lamports};
///
/// #[tokio::main]
/// async fn main() {
///    let private_key = "YOUR_PRIVATE_AS_BASE58_STRING".to_string();
///    let rpc_url = "https://api.mainnet-beta.solana.com/".to_string();
///    let actions = Actions::new(private_key, rpc_url).await;
///
///    let balance = actions.get_balance().await.unwrap();
///    println!("Balance: {}", balance);
///
///    let mint = "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_string();
///    let price = actions.fetch_token_price(mint).await.unwrap();
///    println!("Price: {}", price);
///
///    // Trade 1 WSOL for the given mint
///    let slippage_bps = 100;
///    let tx_id = actions.trade(
///        WSOL.to_string(),
///        sol_to_lamports(1),
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
