# Solana

## Key Features

- **Token Operations**

  - Jupiter swap integration for token trading
  - SPL token transfers and balance checks
  - Token deployment capabilities
  - Price fetching and portfolio management

- **Basic Operations**

  - SOL transfers
  - Balance queries
  - Public key management
  - Portfolio tracking

- **PumpFun Token Features**
  - Token deployment with customizable parameters
  - Buy/sell functionality
  - Price discovery through DexScreener

## Main Tools

The module exposes several key tools:

```
perform_jupiter_swap()    // Execute token swaps via Jupiter
transfer_sol()            // Send SOL to another address
transfer_spl_token()      // Transfer SPL tokens
get_public_key()          // Retrieve signer's public key
get_sol_balance()         // Check SOL balance
get_spl_token_balance()   // Check SPL token balance
deploy_pump_fun_token()   // Deploy on pump.fun
fetch_token_price()       // Get current token prices
get_portfolio()           // Retrieve full portfolio details
search_on_dex_screener()  // search for a ticker/mint
```

## Configuration

The module requires a Solana RPC URL which can be set via the `SOLANA_RPC_URL` environment variable. If not specified, it defaults to the public Solana mainnet RPC endpoint.
