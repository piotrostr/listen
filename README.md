<p align="center">
<img width="1232" alt="example" src="https://github.com/user-attachments/assets/40250e4e-7e03-45c5-9718-c86d0b9537ff" />
<br>
<a href="https://docs.listen-rs/"><img src="https://img.shields.io/badge/docs-API Reference-blue.svg" /></a> &nbsp;
<a href="https://github.com/piotrostr/listen"><img src="https://img.shields.io/github/stars/piotrostr/listen?style=social" /></a>
</p>

## Intro

Listen is a Solana toolkit for building AI Agents with on-chain actions using the [$ARC rig framework](https://github.com/0xPlaygrounds/rig). It provides functionality for:

- Monitoring large transactions on Raydium V4
- Real-time price monitoring with multiple subscription methods
- Various utilities for Solana token trading and management

## Features

- 🔍 Real-time transaction monitoring
- 💱 Multi-DEX swap execution (Pump.fun, Jupiter V6 API or Raydium)
- 🚀 Blazingly fast transactions thanks to Jito MEV bundles
- 📊 Price tracking and metrics
- 🧰 Token management utilities
- 📈 Performance monitoring with Prometheus integration

For complete rundown, see the CLI output of `cargo run` or the [docs](https://docs.listen-rs.com/).

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

Both keypairs are in `solana-keygen` format, array of 64 bytes, 32 bytes private key and 32 bytes public key.

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

## Metrics and Monitoring

Listen includes built-in metrics exposed at `localhost:3030/metrics`. To visualize:

1. Start Prometheus:

```bash
prometheus --config=prometheus.yml
```

2. Access metrics at `localhost:3030/metrics`

## Advanced Usage

### Swap Profiling

Profile swap performance using DTrace:

```bash
./hack/profile-swap.sh
```

## Warning

> [!WARNING]
> Default configuration is set for mainnet with small transactions. Ensure proper configuration for testnet usage and carefully review code before execution.

<p align="center">
<br>
Made with ❤️  by piotrostr
</p>
