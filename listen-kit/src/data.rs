use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;

pub async fn fetch_metadata(
    mint: String,
    rpc_client: &RpcClient,
) -> Result<String> {
    // let metadata = rpc_client.get_account_metadata(&mint).await?;
    // Ok(metadata)
    Ok("metadata".to_string())
}
