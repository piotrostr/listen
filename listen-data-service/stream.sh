#!/bin/bash

START_BLOCK=$(solana slot)
FARTCOIN=9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump

# substreams run solana-accounts-foundational filtered_accounts -t +10 -p filtered_accounts="account:$FARTCOIN"

substreams run \
    ao-solana-dex-trades-pubsub@v0.0.3 tl_solana_dex_trades:map_block \
    --start-block "$START_BLOCK" 
    #-p filtered_accounts="account:$FARTCOIN"


# substreams run ao-solana-dex-trades-pubsub@v0.0.3 tl_solana_dex_trades:map_block --start-block "$START_BLOCK" |
# while read line; do
#     if [[ $line =~ "blockDate" ]]; then
#         ((count++))
#         echo "Trade count: $count"
#     fi
# done


# substreams run \
#     solana_common@v0.2.0 filtered_transactions_without_votes \
#     --start-block "$START_BLOCK"
