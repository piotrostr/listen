#!/bin/bash


# mainnet, careful this is 0.01 sol
cargo run -- swap \
  --dex raydium \
  --input-mint sol \
  --output-mint 2dEwnfrpkmZbKRySNDtMvd4hdzJtjiuW8iLLA6wiCSJs \
  --amm-pool-id 2uNCLciBjNEYzTba2Bg3ZKfWTJVaDtqMPi8Qmb633Bfi \
  --slippage 300 \
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
