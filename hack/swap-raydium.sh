#!/bin/bash


# mainnet, careful this is 0.01 sol
cargo run -- swap \
  --dex raydium \
  --input-mint sol \
  --output-mint CGfFQF8UXh36tpPqKR7ZAoQN9ALjQtbAg7b7KdEQF6Mt \
  --amm-pool-id B1QSDpyybdhVQGJhLXQu4zZApRQt5FzcHJVY48Bb4wp7 \
  --slippage 100 \
  --amount 1000000 \
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
