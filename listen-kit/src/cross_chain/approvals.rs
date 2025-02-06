use anyhow::Result;

pub async fn get_allowance(
    token_address: &str,
    owner_address: &str,
    spender_address: &str,
) -> Result<u128> {
    // Construct the allowance function call data
    let allowance_data = format!(
        "0xdd62ed3e{:0>64}{:0>64}", // allowance(address,address) function selector
        owner_address.trim_start_matches("0x"),
        spender_address.trim_start_matches("0x")
    );

    // Construct the JSON-RPC request
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{
            "to": token_address,
            "data": allowance_data
        }, "latest"],
        "id": 1
    });

    // Make the RPC call
    let client = reqwest::Client::new();
    let res = client
        .post(std::env::var("ETHEREUM_RPC_URL")?)
        .json(&rpc_request)
        .send()
        .await?;

    let response: serde_json::Value = res.json().await?;

    // Parse the response
    let allowance = if let Some(result) = response.get("result") {
        let allowance_hex = result.as_str().unwrap_or("0x0");
        u128::from_str_radix(allowance_hex.trim_start_matches("0x"), 16)
            .unwrap_or(0)
    } else {
        0
    };

    Ok(allowance)
}

pub fn create_approval_transaction(
    token_address: &str,
    spender_address: &str,
    amount: u128,
    from_address: &str,
) -> Result<serde_json::Value> {
    // Construct the approve function call data
    // approve(address,uint256) function selector is 0x095ea7b3
    let amount_hex = format!("{:064x}", amount);
    let approve_data = format!(
        "0x095ea7b3{:0>64}{}",
        spender_address.trim_start_matches("0x"),
        amount_hex
    );

    // Construct the JSON-RPC transaction format
    Ok(serde_json::json!({
        "from": from_address,
        "to": token_address,
        "data": approve_data,
        // Optional fields set to null/None to let the wallet/RPC handle them
        "chainId": null,
        "gas": null,
        "gasPrice": null,
        "value": "0x0"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_get_allowance() {
        // USDC on Arbitrum
        let token = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
        // Random addresses for testing
        let owner = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";
        let spender = "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE";

        // Make sure ETHEREUM_RPC_URL is set for the test
        env::set_var("ETHEREUM_RPC_URL", "https://arb1.arbitrum.io/rpc");

        let result = get_allowance(token, owner, spender).await;
        assert!(result.is_ok());

        let allowance = result.unwrap();
        // Since we're using dummy addresses, allowance should be 0
        assert_eq!(allowance, 0);
    }

    #[tokio::test]
    async fn test_get_allowance_invalid_token() {
        let token = "0x0000000000000000000000000000000000000000";
        let owner = "0x0000000000000000000000000000000000000001";
        let spender = "0x0000000000000000000000000000000000000002";

        env::set_var("ETHEREUM_RPC_URL", "https://arb1.arbitrum.io/rpc");

        let result = get_allowance(token, owner, spender).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
