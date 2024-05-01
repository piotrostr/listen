use std::str::FromStr;

use crate::{constants, types};

use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};

pub fn get_client(url: &str) -> Result<RpcClient, Box<dyn std::error::Error>> {
    let rpc_client =
        RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let signer = rpc_client.get_identity()?;

    println!("Connecting to blocks through {:?}", url);
    println!("Latest blockhash: {:?}", latest_blockhash);
    println!("Signer: {:?}", signer);

    Ok(rpc_client)
}

// Provider provides the data, contains both RPC client that can
// communicate over the REST interface and utilities like getting
// the pricing data from Jupiter
pub struct Provider {
    rpc_client: RpcClient,
}

impl Provider {
    pub fn new(rpc_url: String) -> Provider {
        Provider {
            rpc_client: get_client(rpc_url.as_str()).unwrap(),
        }
    }
    pub fn get_tx(
        &self,
        signature: &str,
    ) -> Result<
        EncodedConfirmedTransactionWithStatusMeta,
        Box<dyn std::error::Error>,
    > {
        let sig = Signature::from_str(signature)?;
        let tx = self.rpc_client.get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(1),
            },
        )?;
        Ok(tx)
    }

    pub async fn get_pricing(
        &self,
        mint: &str,
    ) -> Result<types::PriceResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://price.jup.ag/v4/price?ids={}&vsToken={}",
            mint,
            constants::SOLANA_PROGRAM_ID,
        );
        println!("Getting pricing from: {:?}", url);
        let client = reqwest::Client::new();
        let res = client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await?;
        let data = res.json::<types::PriceResponse>().await?;
        Ok(data)
    }
}
