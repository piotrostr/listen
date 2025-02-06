# Introduction

**Rig Onchain Kit** is a robust framework for building AI-powered applications
that interact natively with blockchain networks. Combining the cognitive
capabilities of large language models with secure blockchain operations, this
toolkit enables developers to create intelligent agents capable of executing
complex on-chain interactions across both Solana and EVM-compatible networks.

At its core, Rig Onchain Kit merges:

- The **`rig-core`** AI agent framework for natural language processing and
  decision-making
- The **`listen`** blockchain library for Solana and EVM transaction
  orchestration
- A **production-ready HTTP service** with real-time streaming capabilities
- Secure **multi-chain wallet management** through Privy integration

The toolkit provides pre-built agents equipped with essential blockchain
operations including token swaps (via Jupiter and Uniswap), asset transfers,
balance queries, and smart contract interactions. Developers can extend
functionality using the `#[tool]` macro system to create custom operations
while maintaining strict security boundaries through the `SignerContext`
architecture.

Key differentiators:

- **Dual-chain First** - Native support for Solana and EVM ecosystems with
  automatic RPC configuration
- **Secure by Design** - Thread-local signer isolation and Privy-based
  authentication for production deployments
- **Real-time Streaming** - SSE-enabled HTTP service handles concurrent user
  sessions with tool call transparency
- **Extensible Tool System** - Combine prebuilt DeFi operations with custom
  logic through macro-driven tool creation
- **Wallet Agnostic** - Supports both local key management and Privy-embedded
  wallets for user-friendly onboarding

Whether building trading assistants, portfolio managers, or DeFi automation
tools, Rig Onchain Kit abstracts blockchain complexity while maintaining full
control over transaction security and model behavior. The included HTTP service
layer enables seamless integration with web frontends while the modular
architecture allows incremental adoption of specific components.
