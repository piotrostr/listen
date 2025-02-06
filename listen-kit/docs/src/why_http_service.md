# Why and how?

While having a possibility of creating local agents is nice, it is all about scale

In order to provide AI Agent powered interactions for the users, it is crucial to be able to expose the agents built with `rig-onchain-kit` as a service, consumable by a frontend

Luckily, the framework comes pre-packaged with a production-ready service for maintaining thousands of simulatenous conversations, each completely encapsulated

To provide a backend for multi-VM platform, with end user wallet management for both EVM and Solana, you can

```sh
git clone https://github.com/piotrostr/listen
cd listen/listen-kit
cp .env.example .env
vi .env # fill in the env vars, include the PRIVY_* variables
cargo run --bin server --features full
```

To find the configuration variables, you can go to your Privy dashboard

> ℹ️ the `PRIVY_VERIFICATION_KEY` has to come in the format of
> `"-----BEGIN PUBLIC KEY-----\n<secret>\n-----END PUBLIC KEY-----"`
> note how there are `'\n'` newline separators, entire secret in a single line

Next chapter outlines how to authenticate your users.
