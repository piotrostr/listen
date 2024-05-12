#!/bin/bash


# mainnet, careful this is 0.01 sol
cargo run -- swap \
  --dex raydium \
  --input-mint sol \
  --output-mint H79qZvpcLfwJC9kiYnZMmwK19Af5FPugfGKqTPDRYP7h \
  --amm-pool-id 5d9QTvkkjsy46WRWxDZY4VzzBD4RUHMsyxukMdJTi8Qo \
  --slippage 1000 \
  --amount $1 \
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
