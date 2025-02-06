# Authentication

After running `cargo run --bin server --features full`, you can make requests
to the backend from a frontend that implements Privy authentication

The service exposed by `rig-onchain-kit` with the `http` feature uses Privy for
frontend to backend authentication

> ⚠️ Be sure to include the same `PRIVY_APP_ID` both in the backend and frontend
> configuration

On your frontend, Privy can be set up as per the documentation
[here](https://docs.privy.io/)

The backend expects the user to have created an embedded wallet and have
delegated access to the application, for both the EVM and Solana embedded
wallets

After completing the authentication, you can use the `getAuthToken` method from
the Privy SDK to authenticate a given user on the backend

Frontend (React example)

```ts
import { usePrivy } from "@privy-io/react-auth";
import { useCallback } from "react";

function Chat() {
  const { getAccessToken } = usePrivy();

  const sendMessage = useCallback(async (userMessage: string) => {
    const body = JSON.stringify({
      prompt: userMessage,
      chat_history: chat_history,
      chain: chatType,
    });

    // post the request to the `rig-onchain-kit` service
    const response = await fetch(config.API_BASE_URL + "/v1/stream", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + (await getAccessToken()),
      },
      body,
    });
  });
}
```

[Full example](https://github.com/piotrostr/listen/blob/main/dashboard/src/hooks/useChat.ts)

On the backend, the token is picked up by middleware and passed onto the the
Privy `WalletManager` implementation, which parses the JWT token sent from the
frontend

```rust
pub fn validate_access_token(
    &self,
    access_token: &str,
) -> Result<PrivyClaims> {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&["privy.io"]);
    validation.set_audience(&[self.privy_config.app_id.clone()]);

    let key = DecodingKey::from_ec_pem(
        self.privy_config.verification_key.as_bytes(),
    )?;

    let token_data =
        decode::<PrivyClaims>(access_token, &key, &validation)
            .map_err(|_| anyhow!("Failed to authenticate"))?;

    Ok(token_data.claims)
}
```

Afterwards, the given user profile is fetched to find the wallets and construct
the `UserSession`, that is later used for finding the signing address to use

This prevents unauthorized access as well as allows to bind the request to
a given user
