# Installation

In order to quickly get set-up, you can add Rig Onchain Kit to your project with

```bash
cargo add rig-onchain-kit --features full
```

Note that full contains both Solana, EVM and http features, where Solana and EVM are available separately

```bash
cargo add rig-onchain-kit --features solana  # less dependencies
```

If you need to add custom tools, with the `#[tool]` macro, be sure to

```bash
cargo add rig-tool-macro
```

In case you are running in a remote environment, should you run into
SSL errors - be sure to include the TLS deps, with

```bash
sudo apt-get update && sudo apt-get install -y \
    ca-certificates \
    openssl \
    libssl3
```
