//! For EVM transactions, approvals are required and for delegated actions,
//! those have to be handled server-side
use anyhow::Result;

pub mod chain_id;
pub mod error;

use chain_id::chain_id_to_ethereum_rpc_url;
use error::ApprovalsError;

pub async fn get_allowance(
    token_address: &str,
    owner_address: &str,
    spender_address: &str,
    chain_id: &str,
) -> Result<u128, ApprovalsError> {
    let rpc_url = chain_id_to_ethereum_rpc_url(chain_id)?;

    // Construct the allowance function call data
    let allowance_data = format!(
        "0xdd62ed3e{:0>64}{:0>64}", // allowance(address,address) function selector
        owner_address.trim_start_matches("0x"),
        spender_address.trim_start_matches("0x")
    );

    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{
            "to": token_address,
            "data": allowance_data
        }, "latest"],
        "id": 1
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
        let allowance_hex = result.as_str().unwrap_or("0x0");
        u128::from_str_radix(allowance_hex.trim_start_matches("0x"), 16).unwrap_or(0)
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
    chain_id: &str,
) -> serde_json::Value {
    // Construct the approve function call data
    // approve(address,uint256) function selector is 0x095ea7b3
    let amount_hex = format!("{:064x}", amount);
    let approve_data = format!(
        "0x095ea7b3{:0>64}{}",
        spender_address.trim_start_matches("0x"),
        amount_hex
    );

    // Construct the JSON-RPC transaction format
    serde_json::json!({
        "from": from_address,
        "to": token_address,
        "data": approve_data,
        "chain_id": chain_id,
        "gas_limit": null,
        "gas_price": null,
        "value": "0x0"
    })
}
