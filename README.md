# listen

Listen to new large transactions on Raydium V4

## Usage

Requires env var of `RPC_URL` with some quota, set the size of buffer for new
transactions and the worker count that process incoming transactions

Listening is over `wss://api.mainnet-beta.solana.com/`, fetching transactions
is over url set in `RPC_URL`

```sh
cargo run -- \
  --listen \
  --worker-count [COUNT] \
  --buffer-size [SIZE]
```

Should yield output as

```txt
Metrics server running on 3030

Connecting to logs for 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8

Connecting to blocks through $RPC_URL
Latest blockhash: BS8s86pQf45V9skwmzWz4GsYyZs6A8gsNWy4i9piV786
Signer: DmFqB5LdQspumdvtHRi4PHKTZENHMw2xRMmSXP6aSkfj

https://solana.fm/tx/3GvmxAxQaWJTNjJnwgwXfP8HNsoasKLRq1wZtYBUEqurPsuFoBUiuV2nfWFKWpvhN2Tt5zVic9fBangFrV2TzSyk: 10.730849095 SOL
https://solana.fm/tx/4aGoJQjRz6P4Ht8BU7k9FY8sQciaDccykr5tDTUXvVQSvHkyX18c9m8RM7LmDd8icmNA4kQyunuDJcQpH36PwMDH: 77.293938152 SOL
https://solana.fm/tx/j7TDeDCds1LYcvnPio9m3v285LSQh4pzeq7K9c7wTWP4deTPnJ7ojTTsuvuKyn3Q74Y23hbrTpyv95Am2RtZHs3: 77.681362228 SOL
```

Automatically, it will send metrics to `localhost:3030/metrics`; to see the
transactions received and processed in Prometheus, run

```sh
prometheus --config=prometheus.yml
```

This will start a server on `localhost:3030/metrics` that contains basic
information of `transactions_received` and `transactions_processed`

In Grafana it looks like this:
<img width="910" alt="image" src="https://github.com/piotrostr/listen/assets/63755291/95668158-9f7d-4cd2-be84-7c2b893d3f5c">
