# Quick Start

First, ensure the [installation](./installation.md) and the [configuration](./configuration.md) steps are completed

Import an agent of choice, along with the `SignerContext` and the local signer struct

```rust
use std::sync::Arc;
use rig_onchain_kit::agent::create_solana_agent;
use rig_onchain_kit::signer::SignerContext;
use rig_onchain_kit::signer::solana::LocalSolanaSigner,
use rig::completion::Prompt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")?;

    let signer = LocalSolanaSigner::new(private_key);

    SignerContext::with_signer(Arc::new(signer), async {
        let agent = create_solana_agent();
        let response = agent.prompt("what is my public key?")?);
        println!("{}", response);
    });

    Ok(())
}
```

For more examples, check out the `examples` directory, you can run each with
`cargo run --example [name]`
