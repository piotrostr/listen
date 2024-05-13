# TODOs

- [x] get the pool details beforehand, wait for the amount out calculation for when jito leader is there
- [ ] track the slot of mint creation and the swap tx execution
- [x] verify the amount out param (slippage)
- [x] ensure re-connect when socket closes
- [ ] move sniper configuration to an object, print on start
- [ ] track the swaps, see if jupiter pricing api has the fresh tokens, otherwise track based on raydium (custom)
  - [ ] save the new holdings into a key-value store like redis
  - [ ] add self initial after a 2x
- [x] add maximum amount of slots of wait
  - [ ] make the bot run replicas in different regions
- [ ] migrate the rpc_client to non-blocking
- [ ] separate the listening and transactions so that it is non-blocking in case of multiple pairs in short span
- [ ] compare entry to available total liquidity
- [ ] benchmark the RPCs a bit more, so far <https://api.mainnet-beta.solana.com/> provides best latency
- [ ] check the liquidity on creation
- [ ] check the top 10 holders balance (red-flag if already weird at launch)
- [ ] check the circulating supply (similar check to previous point)
- [ ] might need to work with larger slippage (20%, 10% does not land many), leaving

```txt
[2024-05-13T15:59:55Z WARN  listen] swap tx: internal error Searcher service did not provide bundle status in time
```
