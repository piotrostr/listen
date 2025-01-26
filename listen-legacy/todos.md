# TODOs (and observations)

- the data shows that some projects burn liquidity and dump straight
  afterwards, even when there is a lot of initial traction, the LP burn does not
  guarantee a good entry since it happens quite late for a lot of projects
- using min amount out 0 is dangerous, as sometimes the pool is not going to be
  updated at the right time and you can send a swap to a rugpulled token, swapping
  with insta -100% pnl

- [x] filter out pump.fun tokens (mostly post-launch dumps)
- [x] get the pool details beforehand, wait for the amount out calculation for when jito leader is there
- [x] get the price of entry based on tx receipt, track the positions in another service
- [x] screen the liquidity burn before listening onto LP event in case the LP
      burn has already happened (JITO bundles, network delays)
- [x] track the slot of mint creation and the swap tx execution
- [x] verify the amount out param (slippage)
- [x] ensure re-connect when socket closes
- [x] move sniper configuration to an object, print on start
- [x] track the swaps, see if jupiter pricing api has the fresh tokens, otherwise track based on raydium (custom)
  - [x] save the new holdings into a key-value store like redis
  - [x] add self initial after a 2x
- [x] add maximum amount of slots of wait (edit: waiting for leader to be JITO is not required)
- [x] migrate the rpc_client to non-blocking
- [x] separate the listening and transactions so that it is non-blocking in case of multiple pairs in short span
- [x] compare entry to available total liquidity
- [x] benchmark the RPCs a bit more, so far <https://api.mainnet-beta.solana.com/> provides best latency
- [x] check the liquidity on creation
- [x] check the top 10 holders balance (red-flag if already weird at launch)
- [x] check the circulating supply (similar check to previous point)
- [x] might need to work with larger slippage (20%, 10% does not land many), leaving

below is displayed sometimes but transaction lands

```txt
[2024-05-13T15:59:55Z WARN  listen] swap tx: internal error Searcher service did not provide bundle status in time
```
