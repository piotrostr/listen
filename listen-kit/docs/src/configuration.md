# Configuration

Rig-onchain-kit uses below environment variables, each is corresponding to
given features, e.g. `--features solana` is going to require the `# solana` env
vars set.

```sh
ANTHROPIC_API_KEY=""

# solana
SOLANA_PRIVATE_KEY=""
SOLANA_RPC_URL=""

# evm
ETHEREUM_PRIVATE_KEY=""
ETHEREUM_RPC_URL=""

# http
PRIVY_APP_ID=""
PRIVY_APP_SECRET=""
PRIVY_VERIFICATION_KEY=""
```

In case the `http` feature is used, the private keys are managed by Privy,
making the `SOLANA_PRIVATE_KEY` and `ETHEREUM_PRIVATE_KEY` no longer required.

The default agents are using Claude under the hood, which maintains the balance
of speed and accuracy, other models might be supported in the future but
currently, Claude is best-in-class
