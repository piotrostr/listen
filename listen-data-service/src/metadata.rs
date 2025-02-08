use crate::{kv_store::RedisKVStore, util::make_rpc_client};
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use serde::{Deserialize, Serialize};
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use std::{str::FromStr, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MplTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub ipfs_metadata: Option<IpfsMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplTokenMetadata {
    pub mint_authority: Option<String>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenMetadata {
    pub mint: String,
    pub mpl: MplTokenMetadata,
    pub spl: SplTokenMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IpfsMetadata {
    pub created_on: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub name: String,
    pub symbol: String,
    pub show_name: Option<bool>,
    pub twitter: Option<String>,
    pub website: Option<String>,
}

pub const IPFS_GATEWAYS: &[&str] = &[
    "https://ipfs.io/ipfs/",
    "https://cloudflare-ipfs.com/ipfs/",
    "https://gateway.pinata.cloud/ipfs/",
];

fn convert_ipfs_uri(uri: &str) -> String {
    if uri.starts_with("ipfs://") {
        uri.replace("ipfs://", "https://ipfs.io/ipfs/")
    } else {
        uri.to_string()
    }
}

pub async fn get_token_metadata(
    kv_store: &Arc<RedisKVStore>,
    mint: &str,
) -> Result<Option<TokenMetadata>> {
    if let Some(metadata) = kv_store.get_metadata(mint).await? {
        Ok(Some(metadata))
    } else {
        match TokenMetadata::fetch_by_mint(mint).await {
            Ok(metadata) => {
                kv_store.insert_metadata(&metadata).await?;
                Ok(Some(metadata))
            }
            Err(e) => {
                eprintln!("Failed to fetch metadata for {}: {}", mint, e);
                Ok(None)
            }
        }
    }
}

// TODO worth also adding SPL
impl TokenMetadata {
    pub async fn fetch_by_mint(mint: &str) -> Result<Self> {
        let mpl_metadata = TokenMetadata::fetch_mpl_by_mint(mint).await?;
        let spl_metadata = TokenMetadata::fetch_spl_by_mint(mint).await?;
        Ok(TokenMetadata {
            mint: mint.to_string(),
            mpl: mpl_metadata,
            spl: spl_metadata,
        })
    }

    pub async fn fetch_spl_by_mint(mint: &str) -> Result<SplTokenMetadata> {
        let rpc_client = make_rpc_client()?;
        let token_pubkey = Pubkey::from_str(mint)?;
        let token_account = rpc_client.get_account_data(&token_pubkey).await?;
        let token_data = Mint::unpack(&token_account)?;
        println!("{:?}", token_data);
        Ok(SplTokenMetadata {
            mint_authority: token_data.mint_authority.map(|p| p.to_string()).into(),
            supply: token_data.supply,
            decimals: token_data.decimals,
            is_initialized: token_data.is_initialized,
            freeze_authority: token_data.freeze_authority.map(|p| p.to_string()).into(),
        })
    }

    pub async fn fetch_mpl_by_mint(mint: &str) -> Result<MplTokenMetadata> {
        let rpc_client = make_rpc_client()?;
        let token_pubkey = Pubkey::from_str(mint)?;

        // Find metadata PDA
        let (metadata_pubkey, _) = Metadata::find_pda(&token_pubkey);

        // Get metadata account data
        let metadata_account = rpc_client.get_account_data(&metadata_pubkey).await?;
        let metadata = Metadata::from_bytes(&metadata_account)?;

        println!("{:?}", metadata);

        let uri = convert_ipfs_uri(&metadata.uri);

        // Create base token metadata
        let mut token_metadata = MplTokenMetadata {
            name: metadata.name.trim_matches(char::from(0)).to_string(),
            symbol: metadata.symbol.trim_matches(char::from(0)).to_string(),
            uri: uri.trim_matches(char::from(0)).to_string(),
            ipfs_metadata: None,
        };

        // Fetch IPFS metadata if available
        let client = reqwest::Client::new();
        if let Ok(response) = client.get(&uri).send().await {
            if let Ok(ipfs_metadata) = response.json::<serde_json::Value>().await {
                println!("{}", serde_json::to_string_pretty(&ipfs_metadata).unwrap());
                token_metadata.ipfs_metadata = Some(serde_json::from_value(ipfs_metadata)?);
            }
        }

        Ok(token_metadata)
    }
}

#[cfg(test)]
mod tests {
    // use crate::kv_store::KVStore;

    use crate::kv_store::KVStore;

    use super::*;

    #[tokio::test]
    async fn test_fetch_mpl_by_mint() {
        let mpl_metadata =
            TokenMetadata::fetch_mpl_by_mint("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
                .await
                .unwrap();
        assert!(mpl_metadata.ipfs_metadata.is_some());
        assert_eq!(mpl_metadata.name, "Fartcoin");
    }

    #[tokio::test]
    async fn test_fetch_mpl_by_mint_2() {
        let mpl_metadata =
            TokenMetadata::fetch_mpl_by_mint("Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump")
                .await
                .unwrap();
        assert!(mpl_metadata.ipfs_metadata.is_some());
        assert_eq!(mpl_metadata.name, "listen-rs");
    }

    #[tokio::test]
    async fn test_fetch_spl_by_mint() {
        let spl_metadata =
            TokenMetadata::fetch_spl_by_mint("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
                .await
                .unwrap();
        println!("{:?}", spl_metadata);
    }

    #[tokio::test]
    async fn test_get_token_metadata() {
        let kv_store = Arc::new(RedisKVStore::new());
        let metadata =
            get_token_metadata(&kv_store, "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
                .await
                .unwrap();
        println!("{:?}", metadata);
    }
}
