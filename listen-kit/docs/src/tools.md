# Tools

Any function can be transformed into a tool by applying a `#[tool]` macro onto it

```rust
#[tool] // <- this is the macro
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Then, when creating the agent, in order to supply the agent with the tool

```rust
let agent = rig::providers::anthropic::Client::from_env()
    .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
    .preamble("you are a friendly calculator")
    .max_tokens(1024)
    .tool(Add) // tool becomes present thanks to the macro
    .build();
```

After that, the model is able to perform actions!

## Built-in tools

> ℹ️ To see all of the currently available tools, you can check
> [Solana](https://github.com/piotrostr/listen/blob/main/listen-kit/src/solana/tools.rs)
> and
> [EVM](https://github.com/piotrostr/listen/blob/main/listen-kit/src/evm/tools.rs)
> toolsets

Throughout `rig-agent-kit` follows an opinionated way of implementing tools:

0. critical: ensuring that tools are called inside of the `SignerContext` block - this
   allows to identify the transaction signer - the owner, exposed into the closure
1. creating a transaction for a given action
2. executing the transaction with the `TransactionSigner` contained by the
   `SignerContext` (more on this in the next chapter)
3. All of the tools return a `Result<T>`

Wrapping the signers is tricky, so `rig-onchain-kit` comes with
helpers, both for EVM and Solana, arriving at this concise end-result

```rust
#[tool]
pub async fn transfer_sol(to: String, amount: u64) -> Result<String> {
    execute_solana_transaction(move |owner| async move {
        create_transfer_sol_tx(&Pubkey::from_str(&to)?, amount, &owner).await
    })
    .await
}
```

This design allows to use different transaction signing and sending methods and
ensuring highly concurrent services using `rig-onchain-kit` work well

## Custom tools

In order to implement extra tools, you can import the helpers along with

```rust
use rig_tool_macro::tool;
use rig_agent_kit::solana::execute_solana_transaction;

use crate::your_package::create_your_custom_tx;

#[tool]
pub async fn custom_tool() -> Result<String> {
    execute_solana_transaction(move |owner| async move {
       // note: the `owner` address/pubkey is available as `String` to consume
       create_your_custom_tx(&owner).await
    })
    .await
}
```

> ⚠️ The tool macro acccepts only the native JSON types, like `string`, `bool`,
> `number` etc, structs and nested types are not supported, so neither
> a `Pubkey` and an `Address` are not allowed, those have to be parsed before
> passing to the corresponding transaction creators
