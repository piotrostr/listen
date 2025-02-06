# SignerContext

The `SignerContext` is a crucial building block of the `rig-onchain-kit` - it
allows to restrict the scope of the signer private key into a thread-local variable

It allows multi-tenancy, where any user chatting with the AI model is able to
maintain the context of their account and their account only in a non-locking
manner

`SignerContext` exposes a `::with_signer()` method, which takes
a `TransactionSigner` trait

It can be used like this

```rust
fn example() {
    SignerContext::with_signer(Arc::new(signer), async {
        // any tool calls inside of this block have the signer passed on
    });
}
```

`rig-onchain-kit` currently supports local signers (the `LocalSolanaSigner`,
`LocalEvmSigner`) as well as the `PrivySigner` for remote signatures

The core methods of the `TransactionSigner` can be implemented with any KMS,
allowing integrations with keys stored inside of HashiCorp Vault, AWS KMS etc,
as well as providers for smart transactions, like Helius or other wallet
management providers, say Magic

High-level interface is as per below snippet

```rust
#[async_trait]
pub trait TransactionSigner: Send + Sync {
    #[cfg(feature = "solana")]
    fn pubkey(&self) -> String;

    #[cfg(feature = "solana")
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String>;

    #[cfg(feature = "evm")]
    fn address(&self) -> String;

    #[cfg(feature = "evm")]
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String>;
}
```

In summary, `SignerContext` combined with `TransactionSigner` provide the
required level of abstraction, where remote keys are safely stored and each
request is processed in its corresponding scope, making `rig-onchain-kit`
a scalable solution

For a production-style implementation with Privy, check out the
`src/http/routes.rs` stream endpoint
