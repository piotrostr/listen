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
    pub ipfs_metadata: Option<serde_json::Value>,
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

fn extract_ipfs_cid(uri: &str) -> Option<String> {
    if uri.starts_with("ipfs://") {
        Some(uri.replace("ipfs://", ""))
    } else if uri.contains("/ipfs/") {
        uri.split("/ipfs/").nth(1).map(|s| s.to_string())
    } else {
        None
    }
}

fn convert_ipfs_uri(uri: &str) -> String {
    if let Some(cid) = extract_ipfs_cid(uri) {
        format!("https://ipfs.io/ipfs/{}", cid)
    } else {
        uri.to_string()
    }
}

pub async fn get_token_metadata(
    kv_store: &Arc<RedisKVStore>,
    mint: &str,
) -> Result<Option<TokenMetadata>> {
    // Try to get from cache first
    if let Some(metadata) = kv_store.get_metadata(mint).await? {
        debug!(mint, "metadata found in cache");
        return Ok(Some(metadata));
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

        let account = token_account.value.context("Token account not found")?;
        let data = &account.data;

        // Check the owner to determine if it's a Token-2022 mint
        let token_2022_program_id =
            Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
                .unwrap();
        let standard_token_program_id =
            Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                .unwrap();

        let token_data = if account.owner == token_2022_program_id {
            warn!(
                mint,
                "detected Token-2022 mint, FIXME: currently missing support"
            );
            // For Token-2022, we still use the same Mint structure for basic fields
            // The structure is the same for the base fields we care about
            Mint::unpack(data)
                .context("failed to unpack Token-2022 mint data")?
        } else if account.owner == standard_token_program_id {
            debug!(mint, "detected standard SPL Token mint");
            Mint::unpack(data).context("failed to unpack mint data")?
        } else {
            return Err(anyhow::anyhow!(
                "Unknown token program owner: {}",
                account.owner
            ));
        };

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
                token_metadata.ipfs_metadata = Some(ipfs_metadata);
            } else {
                warn!(mint, uri, "ipfs response not json");
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
        let kv_store = make_kv_store().await.unwrap();
        let metadata = get_token_metadata(
            &kv_store,
            "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        )
        .await
        .unwrap();
        debug!("{:?}", metadata);
    }

    #[test]
    fn test_extract_ipfs_cid() {
        assert_eq!(
            extract_ipfs_cid("ipfs://QmSomeHash"),
            Some("QmSomeHash".to_string())
        );
        assert_eq!(
            extract_ipfs_cid("https://ipfs.io/ipfs/QmSomeHash"),
            Some("QmSomeHash".to_string())
        );
        assert_eq!(
            extract_ipfs_cid("https://gateway.pinata.cloud/ipfs/QmSomeHash"),
            Some("QmSomeHash".to_string())
        );
        assert_eq!(extract_ipfs_cid("https://example.com/something"), None);
        assert_eq!(
            extract_ipfs_cid(
                "https://gateway.pinata.cloud/ipfs/QmNez6GhGsCYmcW34StMuRw4CWRHZurXmUurQdePV5XcAe"
            ),
            Some("QmNez6GhGsCYmcW34StMuRw4CWRHZurXmUurQdePV5XcAe".to_string())
        );
    }

    #[tokio::test]
    #[ignore = "FIXME: currently missing support"]
    async fn test_spl_2022_mint() {
        let metadata = TokenMetadata::fetch_spl_by_mint(
            "6J7mUbPXcAASzmG4k3umUnT1zaSw97WwduJM2aKJCeiF",
        )
        .await
        .unwrap();

        println!("{:?}", metadata);
        assert!(metadata.is_initialized);
    }
}
