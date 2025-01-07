# listen

Listen to new large transactions on Raydium V4, swap using Pump.fun, Jupiter V6 API or directly with Raydium (crucial for new
listings)

Monitor prices real-time, multiple subscription ways and providers (PubSub,
webhooks..)

This tool has accidentally evolved to something of a sort of a Solana
swiss-knife, has a bunch of stuff useful when trading shitcoins on Raydium and
Pump.fun and some utilities like generating a custom wallet pubkey, mine is
`fuckTY...`

There are some methods for sniping both platforms, 'sweeping' all of the bought
tokens (selling hundreds of bought tokens), closing all of the associated token
accounts (if you snipe ~500 tokens you can close those accounts later and
retrieve like 1 sol), check out the functionalities in the outline below

```txt
$ listen
Usage: listen [OPTIONS] <COMMAND>

Commands:
  close-token-accounts
  pump-service
  grab-metadata
  sell-pump
  bump-pump
  sweep-pump
  snipe-pump
  buy-pump-token
  generate-custom-address
  ata
  spl-stream
  monitor-mempool
  seller-service
  checker-service
  checks
  blockhash
  listen-for-sol-pooled
  buyer-service
  track-position
  top-holders
  monitor-leaders
  monitor-slots
  price
  bench-rpc
  priority-fee
  tx
  listen
  listen-for-burn
  listener-service
  snipe
  wallet
  parse-pool
  swap
  help                 Print this message or the help of the given subcommand(s)

Options:
  -u, --url <URL>                 [default: https://api.mainnet-beta.solana.com]
  -w, --ws-url <WS_URL>           [default: wss://api.mainnet-beta.solana.com]
  -k, --keypair-path <KEYPAIR_PATH>
      --tokio-console
  -h, --help                      Print help
  -V, --version                   Print version
```

Be careful as the default usage was on mainnet with small txs, be sure to set
the URLs and signer keypair to testnet, read the code too so that you don't
mess anything up

## Requirements

0. Install Rust
1. Install `protoc`, `build-essential`, `pkg-config`, `libssl-dev`
2. Copy over `.env.example` into `.env`, plug your RPCs (otherwise uses solana
   default public RPCs)
3. Copy over `auth.json` - JITO authentication keypair and `fund.json` - the
   keypair with some SOL to fund the endeavour
4. Enable Rust Nightly and `cargo build --release`
5. Run the `./run-systemd-services.sh` to run the microservices

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
src="https://github.com/piotrostr/listen/assets/63755291/95668158-9f7d-4cd2-be84-7c2b893d3f5c">

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
