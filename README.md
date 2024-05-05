# listen

Listen to new large transactions on Raydium V4

Swap using Jupiter V6 API

Be careful as the default usage was on mainnet with small txs, be sure to set
the URLs and signer keypair to testnet

## Usage

### Listening on new swaps

Requires env var of `RPC_URL` with some quota, set the size of buffer for new
transactions and the worker count that process incoming transactions

Listening is over `wss://api.mainnet-beta.solana.com/`, fetching transactions
uses the url set in `RPC_URL`

```sh
cargo run -- listen \
  --worker-count [COUNT] \
  --buffer-size [SIZE]
```

Should yield output as

```txt
Metrics server running on 3030

Connecting to logs for 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8

Connecting to blocks through [RPC_URL]
Latest blockhash: BS8s86pQf45V9skwmzWz4GsYyZs6A8gsNWy4i9piV786
Signer: DmFqB5LdQspumdvtHRi4PHKTZENHMw2xRMmSXP6aSkfj

https://solana.fm/tx/3Gvmx...: 10.730849095 SOL
https://solana.fm/tx/4aGoJ...: 77.293938152 SOL
https://solana.fm/tx/j7TD...: 77.681362228 SOL
```

Automatically, it will send metrics to `localhost:3030/metrics`; to see the
transactions received and processed in Prometheus, run

```sh
prometheus --config=prometheus.yml
```

This will start a server on `localhost:3030/metrics` that contains metrics
(`transactions_received` and `transactions_processed`)

In Grafana it looks like this:
<img
width="910"
alt="image"
src="https://github.com/piotrostr/listen/assets/63755291/95668158-9f7d-4cd2-be84-7c2b893d3f5c"

>

### Swapping

The account used to sign the transaction is by default the
`~/.config/solana/id.json`, but it is possible to specify the path using
`--keypair-path [PATH]`, the account has to be generated or imported using
`solana-keygen` executable that ships with remaining the Solana SDK

Slippage can also be adjusted using `--slippage [BPS]`, e.g. `--slippage 50`
for 0.5% slippage, default is 50, dynamic slippage and retries is in the prod
roadmap

```sh
cargo run -- swap \
  --input-mint sol \
  --output-mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
  --amount 10000000
```

Amount is in lamports (or SPL-Token decimals), leave blank to automatically
swap the entire balance of the input mint

There is an optional parameter `-y` or `--yes` which skips the confirmation on
the user side

#### Swap profiling

There is a utility for profiling a single swap using `DTrace` and
`stackcollapse.pl` ([Repository](https://github.com/brendangregg/FlameGraph)),
replace your home directory (mine is `/Users/piotrostr`) to point to the right
`id.json` keypair

```sh
./hack/profile-swap.sh
```

The `stackcollapse.pl` can be installed through

```
gh repo clone brendangregg/FlameGraph && \
  sudo cp FlameGraph/stackcollapse.pl /usr/local/bin && \
  sudo cp FlameGraph/flamegraph.pl /usr/local/bin
```

yielding

<img width="1210" alt="image" src="https://github.com/piotrostr/listen/assets/63755291/699405b7-adf0-448b-89c1-ba71152dc72b">


