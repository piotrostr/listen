//! For EVM transactions, approvals are required and for delegated actions,
//! those have to be handled server-side
use anyhow::Result;

pub mod chain_id;
pub mod error;

pub use chain_id::*;
pub use error::*;

pub const MAX_APPROVAL_AMOUNT: &str =
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

pub const MAX_APPROVAL_AMOUNT_0X: &str =
    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

pub async fn get_allowance(
    token_address: &str,
    owner_address: &str,
    spender_address: &str,
    chain_id: &str,
) -> Result<String, ApprovalsError> {
    let rpc_url = chain_id_to_ethereum_rpc_url(chain_id)?;

    // Construct the allowance function call data
    let allowance_data = format!(
        "0xdd62ed3e{:0>64}{:0>64}", // allowance(address,address) function selector
        owner_address.trim_start_matches("0x"),
        spender_address.trim_start_matches("0x")
    );

    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [{
            "to": token_address,
            "data": allowance_data
        }, "latest"],
    });

    let client = reqwest::Client::new();
    let res = client
        .post(rpc_url)
        .json(&rpc_request)
        .send()
        .await
        .map_err(ApprovalsError::FailedToGetAllowance)?;

    let response: serde_json::Value = res
        .json()
        .await
        .map_err(ApprovalsError::FailedToGetAllowance)?;

    // Parse the response
    let allowance = if let Some(result) = response.get("result") {
        result.as_str().unwrap_or("0x0").to_string()
    } else {
        "0x0".to_string()
    };

    Ok(allowance)
}

pub async fn estimate_gas_params(
    token_address: &str,
    spender_address: &str,
    from_address: &str,
    chain_id: &str,
) -> Result<(u64, u64), ApprovalsError> {
    let rpc_url = chain_id_to_ethereum_rpc_url(chain_id)?;
    let client = reqwest::Client::new();

    // Construct approval data for gas estimation
    let approve_data = format!(
        "0x095ea7b3{:0>64}{}",
        spender_address.trim_start_matches("0x"),
        MAX_APPROVAL_AMOUNT
    );

    // Estimate gas limit
    let gas_estimate_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": [{
            "from": from_address,
            "to": token_address,
            "data": approve_data,
            "value": "0x0"
        }, "latest"],
        "id": 1
    });

    let res = client
        .post(&rpc_url)
        .json(&gas_estimate_request)
        .send()
        .await
        .map_err(|e| ApprovalsError::FailedToEstimateGas(e.to_string()))?;

    let response: serde_json::Value = res
        .json()
        .await
        .map_err(|e| ApprovalsError::FailedToEstimateGas(e.to_string()))?;

    let gas_limit = if let Some(result) = response.get("result") {
        u64::from_str_radix(
            result.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16,
        )
        .unwrap_or(21000)
    } else {
        21000 // fallback gas limit
    };

    // Get current gas price
    let gas_price_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    });

    let res = client
        .post(&rpc_url)
        .json(&gas_price_request)
        .send()
        .await
        .map_err(|e| ApprovalsError::FailedToEstimateGas(e.to_string()))?;

    let response: serde_json::Value = res
        .json()
        .await
        .map_err(|e| ApprovalsError::FailedToEstimateGas(e.to_string()))?;

    let gas_price = if let Some(result) = response.get("result") {
        u64::from_str_radix(
            result.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16,
        )
        .unwrap_or(1_000_000_000) // 1 gwei fallback
    } else {
        1_000_000_000 // 1 gwei fallback
    };

    Ok((gas_limit, gas_price))
}

pub async fn create_approval_transaction(
    token_address: &str,
    spender_address: &str,
    from_address: &str,
    chain_id: &str,
) -> Result<serde_json::Value, ApprovalsError> {
    // Get gas parameters
    let (gas_limit, gas_price) =
        estimate_gas_params(token_address, spender_address, from_address, chain_id).await?;

    let approve_data = format!(
        "0x095ea7b3{:0>64}{}",
        spender_address.trim_start_matches("0x"),
        MAX_APPROVAL_AMOUNT
    );

    // Construct the JSON-RPC transaction format
    let res = serde_json::json!({
        "from": from_address,
        "to": token_address,
        "data": approve_data,
        "chainId": format!("0x{:x}", chain_id.parse::<u64>().map_err(|e| ApprovalsError::InvalidChainId(e.to_string()))?),
        "gasLimit": format!("0x{:x}", gas_limit),
        "gasPrice": format!("0x{:x}", gas_price),
        "value": "0x0"
    });

    // TODO debug instead of info
    tracing::info!("Approval transaction: {:?}", res);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_allowance() {
        let allowance = get_allowance(
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770",
            "0x1231deb6f5749ef6ce6943a275a1d3e7486f4eae",
            "8453",
        )
        .await;
        println!("Allowance: {:?}", allowance);
        assert!(allowance.is_ok());
        if let Ok(value) = allowance {
            assert_ne!(value, "0x0");
        }
    }
}
