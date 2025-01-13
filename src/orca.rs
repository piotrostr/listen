use log::{debug, error};
use std::cmp::min;
use std::error::Error;

use anchor_lang::AnchorDeserialize;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

#[derive(AnchorDeserialize, Debug)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

#[derive(AnchorDeserialize, Debug)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}

pub async fn get_whirpool(
    rpc_client: &RpcClient,
    pool_pubkey: Pubkey,
) -> Result<Whirlpool, Box<dyn Error>> {
    let raw_account = rpc_client
        .get_account_with_config(
            &pool_pubkey,
            RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
        )
        .await?;

    let account_data = raw_account.value.ok_or("No account data")?.data;

    // Print the length of the account data
    debug!("Account data length: {}", account_data.len());

    // Attempt to deserialize, but print more info if it fails
    match Whirlpool::try_from_slice(&account_data[8..]) {
        Ok(whirlpool) => Ok(whirlpool),
        Err(e) => {
            error!("Deserialization error: {:?}", e);
            debug!(
                "First few bytes of account data: {:?}",
                &account_data[..min(32, account_data.len())]
            );
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::SOLANA_PROGRAM_ID;

    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_whirlpool() {
        let rpc_client =
            RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let pool_pubkey =
            Pubkey::from_str("26nkKSE4YbYASbnvjSnrKkFxcJkfr7RxQ5tTPSd4jSoY")
                .expect("pubkey");
        let whirlpool = get_whirpool(&rpc_client, pool_pubkey)
            .await
            .expect("whirlpool");

        assert_eq!(whirlpool.fee_rate, 3000);
        assert_eq!(whirlpool.protocol_fee_rate, 1300);
        assert_eq!(whirlpool.token_mint_a, SOLANA_PROGRAM_ID);
        assert_eq!(
            whirlpool.token_mint_b.to_string(),
            "CTJf74cTo3cw8acFP1YXF3QpsQUUBGBjh2k2e8xsZ6UL" // $neiro
        );
    }
}
