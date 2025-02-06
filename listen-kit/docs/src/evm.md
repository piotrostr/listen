# EVM

## Key Features

- **Token Operations**

  - Uniswap integration for token swaps
  - ERC20 token transfers and allowance management
  - Token balance checks
  - Router approval verification

- **Basic Operations**
  - ETH transfers
  - Balance queries
  - Wallet address management
  - Gas estimation and transaction handling

## Main Tools

The module exposes several key tools:

```rust
verify_swap_router_has_allowance()  // Check DEX trading permissions
approve_token_for_router_spend()    // Approve tokens for trading
trade()                             // Execute token swaps via Uniswap
transfer_eth()                      // Send ETH to another address
transfer_erc20()                    // Transfer ERC20 tokens
wallet_address()                    // Get current wallet address
get_eth_balance()                   // Check ETH balance
get_erc20_balance()                 // Check ERC20 token balance
```

## Configuration

The module requires an Ethereum RPC URL which can be set via the `ETHEREUM_RPC_URL` environment variable. It supports multiple EVM-compatible chains through provider configuration.
