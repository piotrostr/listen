# listen-kit

Blazingly fast actions for AI Agents

## Quick Start

```rust
use listen_kit::{actions::Actions, util::env, constants::WSOL};
use solana_sdk::native_token::sol_to_lamports;

#[tokio::main]
async fn main() {
   let actions = Actions::new(env("SOLANA_PRIVATE_KEY"), env("SOLANA_RPC_URL"));

   let balance = actions.get_balance().await.unwrap();
   println!("Balance: {}", balance);

   let mint = "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump".to_string();
   let price = actions.fetch_token_price(mint.clone()).await.unwrap();
   println!("Price: {}", price);

   // Trade 0.01 SOL for $Fartcoin
   let slippage_bps = 100;
   let tx_id = actions.trade(
       WSOL.to_string(),
       sol_to_lamports(0.01),
       mint,
       slippage_bps
   ).await.unwrap();
   println!("Signature: {}", tx_id);
}
```
