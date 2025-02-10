use crate::de::*;
use crate::{kv_store::RedisKVStore, util::make_rpc_client};
use anyhow::{Context, Result};
use mpl_token_metadata::accounts::Metadata;
use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Mint;
use std::{str::FromStr, sync::Arc};
use tracing::{debug, warn};

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
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_string_or_object")]
    pub created_on: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_string_or_object")]
    pub description: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_string_or_object")]
    pub image: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_object")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_object")]
    pub symbol: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_bool")]
    pub show_name: Option<bool>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_string_or_object")]
    pub twitter: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_string_or_object")]
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
    if kv_store.has_metadata(mint).await? {
        debug!(mint, "metadata already exists");
        return kv_store.get_metadata(mint).await;
    }

    match TokenMetadata::fetch_by_mint(mint).await {
        Ok(metadata) => {
            kv_store
                .insert_metadata(&metadata)
                .await
                .context("failed to insert metadata")?;

            Ok(Some(metadata))
        }
        Err(e) => Err(e),
    }
}

impl TokenMetadata {
    pub async fn fetch_by_mint(mint: &str) -> Result<Self> {
        let mpl_metadata = TokenMetadata::fetch_mpl_by_mint(mint)
            .await
            .unwrap_or_default();
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
        let token_account = rpc_client
            .get_account_with_commitment(
                &token_pubkey,
                CommitmentConfig::processed(),
            )
            .await
            .context("failed to get token account")?;

        let data = token_account.value.context("Token account not found")?.data;

        let token_data =
            Mint::unpack(&data).context("failed to unpack mint data")?;

        debug!(mint, "spl metadata fetch ok");

        Ok(SplTokenMetadata {
            mint_authority: token_data
                .mint_authority
                .map(|p| p.to_string())
                .into(),
            supply: token_data.supply,
            decimals: token_data.decimals,
            is_initialized: token_data.is_initialized,
            freeze_authority: token_data
                .freeze_authority
                .map(|p| p.to_string())
                .into(),
        })
    }

    pub async fn fetch_mpl_by_mint(mint: &str) -> Result<MplTokenMetadata> {
        let rpc_client =
            make_rpc_client().context("failed to make rpc client")?;
        let token_pubkey =
            Pubkey::from_str(mint).context("failed to parse mint")?;

        // Find metadata PDA
        let (metadata_pubkey, _) = Metadata::find_pda(&token_pubkey);
        debug!(
            mint,
            metadata_pubkey = metadata_pubkey.to_string(),
            "attempting to fetch MPL metadata"
        );

        // Get metadata account data
        let metadata_account = rpc_client
            .get_account_with_commitment(
                &metadata_pubkey,
                CommitmentConfig::processed(),
            )
            .await
            .context(format!(
                "failed to get metadata account: {}",
                metadata_pubkey
            ))?;

        let data = metadata_account
            .value
            .context(format!(
                "Metadata account not found: token: {} mpl pda: {}",
                token_pubkey, metadata_pubkey
            ))?
            .data;

        debug!(mint, data_len = data.len(), "got metadata account data");

        let metadata = match Metadata::from_bytes(&data) {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    mint,
                    error = e.to_string(),
                    "failed to parse metadata, trying alternative parsing"
                );
                return Err(e.into());
            }
        };

        debug!(
            mint,
            name = metadata.name,
            symbol = metadata.symbol,
            uri = metadata.uri,
            "parsed metadata successfully"
        );

        let uri = convert_ipfs_uri(&metadata.uri)
            .trim_matches(char::from(0))
            .to_string();

        // Create base token metadata
        let mut token_metadata = MplTokenMetadata {
            name: metadata.name.trim_matches(char::from(0)).to_string(),
            symbol: metadata.symbol.trim_matches(char::from(0)).to_string(),
            uri: uri.clone(),
            ipfs_metadata: None,
        };

        // Fetch IPFS metadata if available
        let client = reqwest::Client::new();
        if let Ok(response) = client.get(&uri).send().await {
            if let Ok(ipfs_metadata) =
                response.json::<serde_json::Value>().await
            {
                debug!(mint, uri, "ipfs fetch ok");
                token_metadata.ipfs_metadata = Some(
                    serde_json::from_value(ipfs_metadata)
                        .context("failed to parse ipfs metadata")?,
                );
            } else {
                warn!(mint, uri, "ipfs fetch failed");
            }
        } else {
            warn!(mint, uri, "ipfs fetch failed");
        }

        Ok(token_metadata)
    }
}
#[cfg(test)]
mod tests {
    use crate::util::make_kv_store;

    use super::*;

    #[tokio::test]
    async fn test_fetch_mpl_by_mint() {
        let mpl_metadata = TokenMetadata::fetch_mpl_by_mint(
            "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        )
        .await
        .unwrap();
        assert!(mpl_metadata.ipfs_metadata.is_some());
        assert_eq!(mpl_metadata.name, "Fartcoin ");
    }

    #[tokio::test]
    async fn test_fetch_mpl_by_mint_2() {
        let mpl_metadata = TokenMetadata::fetch_mpl_by_mint(
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump",
        )
        .await
        .unwrap();
        assert!(mpl_metadata.ipfs_metadata.is_some());
        assert_eq!(mpl_metadata.name, "listen-rs");
    }

    #[tokio::test]
    async fn test_fetch_mpl_by_mint_3() {
        let mpl_metadata = TokenMetadata::fetch_mpl_by_mint(
            "EfAynhvukY3nGyWEYhDSijDT9NnHA4o7NXDUXcrMpump",
        )
        .await
        .unwrap();

        assert_eq!(mpl_metadata.name, "Parallel AI");
        assert_eq!(mpl_metadata.symbol, "PAI");
    }

    #[tokio::test]
    async fn test_fetch_spl_by_mint() {
        let spl_metadata = TokenMetadata::fetch_spl_by_mint(
            "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        )
        .await
        .unwrap();
        debug!("{:?}", spl_metadata);
    }

    #[tokio::test]
    async fn test_get_token_metadata() {
        let kv_store = make_kv_store().unwrap();
        let metadata = get_token_metadata(
            &kv_store,
            "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        )
        .await
        .unwrap();
        debug!("{:?}", metadata);
    }

    // Add a new test for fetch_by_mint that shows the complete behavior
    #[tokio::test]
    async fn test_fetch_by_mint_no_mpl() {
        let metadata = TokenMetadata::fetch_by_mint(
            "EfAynhvukY3nGyWEYhDSijDT9NnHA4o7NXDUXcrMpump",
        )
        .await
        .unwrap();

        // Should have default MPL metadata
        assert_eq!(metadata.mpl.name, "");
        assert_eq!(metadata.mpl.symbol, "");
        assert_eq!(metadata.mpl.uri, "");
        assert!(metadata.mpl.ipfs_metadata.is_none());

        // But should still have SPL metadata
        assert!(metadata.spl.is_initialized);
    }
}
