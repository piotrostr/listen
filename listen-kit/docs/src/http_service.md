# HTTP Service

The `rig-onchain-kit` provides a production-ready HTTP service that enables:

- Server-Sent Events (SSE) streaming for real-time AI agent responses
- Multi-chain support (EVM and Solana) through feature flags
- User authentication and wallet management via Privy
- Concurrent handling of multiple chat sessions

## Core Endpoints

The service exposes these main endpoints:

```
POST /v1/stream   - Stream AI agent responses
GET  /v1/auth     - Verify authentication status
GET  /healthz     - Health check endpoint
```

### Streaming Endpoint

The `/v1/stream` endpoint accepts:

```typescript
{
  prompt: string,
  chat_history: Message[],
  chain: "solana" | "evm" | "pump" // Chain selection
}
```

It returns a Server-Sent Events stream containing:

```typescript
{
  type: "Message" | "ToolCall" | "Error",
  content: {
    // For Message: string with AI response
    // For ToolCall: { name: string, result: string }
    // For Error: error message string
  }
}
```

## Features

### Chain Selection

The service supports multiple blockchain environments through feature flags:

```rust
// Select agent based on chain parameter
match request.chain.as_deref() {
    #[cfg(feature = "solana")]
    Some("solana") => state.solana_agent.clone(),
    #[cfg(feature = "evm")]
    Some("evm") => state.evm_agent.clone(),
    // ...
}
```

### Concurrent Sessions

The service handles multiple simultaneous chat sessions using Tokio channels and tasks:

```rust
let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(32);
spawn_with_signer(signer, || async move {
    // Handle individual chat session
}).await;
```

### Keep-alive & Retry Logic

The SSE implementation includes built-in keep-alive and retry mechanisms:

```rust
sse::Sse::from_infallible_receiver(rx)
    .with_keep_alive(Duration::from_secs(15))
    .with_retry_duration(Duration::from_secs(10))
```

## Configuration

The service is configured through the `AppState` which manages:

- Chain-specific AI agents
- Wallet manager instance
- Authentication settings

```rust
let state = AppState::builder()
    .with_wallet_manager(wallet_manager)
    .with_solana_agent(solana_agent)    // Optional
    .with_evm_agent(evm_agent)          // Optional
    .build()?;
```
