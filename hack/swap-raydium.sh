#!/bin/bash


# mainnet, careful this is 0.01 sol
cargo run -- swap \
  --dex raydium \
  --input-mint ukHH6c7mMyiWCf1b9pnWe25TSpkDDt3H5pQZgZ74J82 \
  --output-mint sol \
  --amm-pool-id DSUvc5qf5LJHHV5e2tD184ixotSnCnwj7i4jJa4Xsrmt \
  --slippage 100 \
  --yes

exit

# testnet 
cargo run -- swap \
  --dex raydium \
  --testnet \
  --input-mint GfmdKWR1KrttDsQkJfwtXovZw9bUBHYkPAEwB6wZqQvJ \
  --output-mint 2SiSpNowr7zUv5ZJHuzHszskQNaskWsNukhivCtuVLHo \
  --amm-pool-id BbZjQanvSaE9me4adAitmTTaSgASvzaVignt4HRSM7ww \
  --amount 100000000
