<p align="center">
<img src="frontend/public/listen-more.png" width="25%" />
<br />
</p>
<p align="center">
<a href="https://docs.listen-rs.com/"><img src="https://img.shields.io/badge/docs-API-blue.svg" /></a> &nbsp;
<a href="https://github.com/piotrostr/listen"><img src="https://img.shields.io/github/stars/piotrostr/listen?style=social" /></a>
<a href=""><img src="https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust" /></a>
</p>

<p align="center">
<code>listen</code> is a Solana Swiss-Knife toolkit for algorithmic trading
</p>

## Features

- 🔍 Real-time transaction monitoring
- 💱 Multi-DEX swap execution (Pump.fun, Jupiter V6 API or Raydium)
- 🚀 Blazingly fast transactions thanks to Jito MEV bundles
- 📊 Price tracking and metrics
- 🧰 Token management utilities
- 📈 Performance monitoring with Prometheus integration

And more!

It works plug'n'play with [$arc rig
framework](https://github.com/0xPlaygrounds/rig) framework allowing AI Agents
interact with the Solana blockchain, see example:
[src/agent.rs](https://github.com/piotrostr/listen/blob/main/src/agent.rs) and
the output [image](https://github.com/piotrostr/listen/blob/main/example.png).

For complete rundown of features, check out the CLI output of `cargo run` or the
[documentation](https://docs.listen-rs.com/).

This repository contains some miscellanous tools for grabbing data from bullx/gmgn.ai/pump.fun unofficial APIs, in `bullx`, `watcher`, `pump-ts` and an analysis module, all which might be useful, though the core of the library is located in the `src` directory.

## Requirements

1. **System Dependencies**

   - Rust (with nightly toolchain)
   - `protoc`
   - `build-essential`
   - `pkg-config`
   - `libssl-dev`

2. **Configuration**
   - Copy `.env.example` to `.env`
   - Set up `auth.json` for JITO authentication (optional, gRPC HTTP/2.0 searcher client)
   - Populate `fund.json`

Both keypairs are in `solana-keygen` format, array of 64 bytes, 32 bytes
private key and 32 bytes public key.

## Quick Start

```bash
# Install dependencies
sudo apt install protoc build-essential pkg-config libssl-dev

# Build
cargo build --release

# Run services
./run-systemd-services.sh
```

## Usage Examples

### Transaction Monitoring

```bash
cargo run -- listen \
  --worker-count [COUNT] \
  --buffer-size [SIZE]
```

### Token Swapping

```bash
cargo run -- swap \
  --input-mint sol \
  --output-mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
  --amount 10000000
```

> [!WARNING]
> Default configuration is set for mainnet with small transactions. Ensure proper configuration for testnet usage and carefully review code before execution.

## Metrics and Monitoring

Listen includes built-in metrics exposed at `localhost:3030/metrics`. To visualize:

1. Start Prometheus:

```bash
prometheus --config=prometheus.yml
```

2. Access metrics at `localhost:3030/metrics`

Grafana should show something like this

<img
width="910"
alt="image"
src="https://github.com/piotrostr/listen/assets/63755291/95668158-9f7d-4cd2-be84-7c2b893d3f5c">

## Advanced Usage

### Swap Profiling

The `stackcollapse.pl` can be installed through

```sh
gh repo clone brendangregg/FlameGraph && \
  sudo cp FlameGraph/stackcollapse.pl /usr/local/bin && \
  sudo cp FlameGraph/flamegraph.pl /usr/local/bin
```

Profile swap performance using DTrace to produce a flamegraph:

```bash
./hack/profile-swap.sh
```

<img width="1210" alt="image" src="https://github.com/piotrostr/listen/assets/63755291/699405b7-adf0-448b-89c1-ba71152dc72b">

