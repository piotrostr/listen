use super::error::ApprovalsError;

pub fn caip2_to_chain_id(caip2: &str) -> Result<&str, ApprovalsError> {
    let chain_id = caip2
        .split(':')
        .last()
        .ok_or(ApprovalsError::InvalidCaip2(caip2.to_string()))?;
    Ok(chain_id)
}

pub fn chain_id_to_ethereum_rpc_url(chain_id: &str) -> Result<String, ApprovalsError> {
    let alchemy_api_key =
        std::env::var("ALCHEMY_API_KEY").map_err(|_| ApprovalsError::FailedToGetAlchemyApiKey)?;
    match chain_id {
        "1" => Ok(format!(
            "https://eth-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        )),
        "56" => Ok(format!(
            "https://bnb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        )),
        "137" => Ok(format!(
            "https://polygon-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        )),
        "42161" => Ok(format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        )),
        "8453" => Ok(format!(
            "https://base-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        )),
        _ => Err(ApprovalsError::UnsupportedChainId(chain_id.to_string())),
    }
}

pub fn caip2_to_ethereum_rpc_url(caip2: &str) -> Result<String, ApprovalsError> {
    let chain_id = caip2_to_chain_id(caip2)?;
    chain_id_to_ethereum_rpc_url(chain_id)
}
