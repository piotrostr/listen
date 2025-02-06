# Cross-chain

## Key Features

- **Cross-chain Swaps & Bridges**

  - Seamless token swaps between Solana and EVM chains
  - Bridge functionality for moving assets across chains
  - Quote system for cost estimation
  - Support for both native and wrapped tokens

- **Token Operations**

  - ERC20 approval management for cross-chain bridges
  - Token allowance verification
  - Automatic address resolution based on chain type

## Main Tools

The module exposes several key tools for cross-chain operations:

```rust
get_multichain_quote()  // Get cost estimate for cross-chain swaps
multichain_swap()       // Execute cross-chain token swaps/bridges
check_approval()        // Verify ERC20 token approvals
approve_token()         // Approve ERC20 tokens for bridge contracts
```

## Configuration

The module requires an Ethereum RPC URL which can be set via the
`ETHEREUM_RPC_URL` environment variable. It supports multiple EVM-compatible
chains through provider configuration.

## Important Notes

1. **Token Decimals**: Amount parameters must account for token decimals:

   - USDC: 6 decimals (1 USDC = 1000000)
   - SOL: 9 decimals (1 SOL = 1000000000)
   - ETH: 18 decimals (1 ETH = 1000000000000000000)

2. **Gas Requirements**: Users must have native tokens (SOL/ETH) on both chains
   to cover gas fees, unless using sponsored transactions (coming soon).

3. **Token Identification**: Tokens can be specified using:

   - Symbol (e.g., "USDC")
   - Solana public key
   - EVM contract address

4. **Chain Support**: Currently running Solana and Arbitrum, EVM is not fully
   multi-tenant, need some tweaks. Full coverage on the way
