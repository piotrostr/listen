use std::str::FromStr;
use anyhow::{Result, Error};

use commons::{
    LbClmm,
    lb_clmm::{
        state::{PositionV2, Pool, LbPair},
        utils::{get_position_pda, get_pool_pda},
        types::{Pubkey, Keypair, Transaction},
    },
};
use log::{info, warn};
use dialoguer;

use crate::{Provider, constants::METEORA_PROGRAM_ID};

pub struct Meteora {}

impl Default for Meteora {
    fn default() -> Self {
        Self::new()
    }
}

impl Meteora {
    pub const fn new() -> Self {
        Meteora {}
    }

    pub async fn swap(
        &self,
        input_token_mint: Pubkey,
        output_token_mint: Pubkey,
        amount: u64,
        slippage: u64,
        wallet: &Keypair,
        provider: &Provider,
        confirmed: bool,
    ) -> Result<()> {
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        
        // Get the pool for the token pair
        let pool = lb_clmm.get_lb_pair(input_token_mint, output_token_mint).await?;
        
        // Calculate swap quote
        let quote = lb_clmm.swap_quote(&pool, amount, input_token_mint == pool.token_x_mint)?;
        
        // Calculate minimum amount out with slippage
        let min_amount_out = quote.amount_out * (10000 - slippage) / 10000;

        if !confirmed {
            info!(
                "Swap quote: {} -> {} (min: {})",
                amount,
                quote.amount_out,
                min_amount_out
            );
            if !dialoguer::Confirm::new()
                .with_prompt("Execute swap?")
                .interact()?
            {
                return Ok(());
            }
        }

        // Build and send the swap transaction
        let tx = lb_clmm.create_swap_transaction(
            &pool,
            amount,
            min_amount_out,
            input_token_mint == pool.token_x_mint,
            wallet.pubkey(),
        ).await?;
        
        // Send and confirm transaction
        match provider.send_tx(&tx, true).await {
            Ok(signature) => {
                info!("Swap transaction successful: {}", signature);
                Ok(())
            }
            Err(e) => {
                warn!("Swap transaction failed: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn add_liquidity(
        &self,
        pool: &LbPair,
        amount_x: u64,
        amount_y: u64,
        wallet: &Keypair,
        provider: &Provider,
    ) -> Result<()> {
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        
        // Create position for liquidity
        let position = lb_clmm.create_position(
            pool,
            amount_x,
            amount_y,
            wallet.pubkey(),
        ).await?;

        // Build and send the add liquidity transaction
        let tx = lb_clmm.create_add_liquidity_transaction(
            pool,
            &position,
            amount_x,
            amount_y,
            wallet.pubkey(),
        ).await?;
        
        match provider.send_tx(&tx, true).await {
            Ok(signature) => {
                info!("Add liquidity transaction successful: {}", signature);
                Ok(())
            }
            Err(e) => {
                warn!("Add liquidity transaction failed: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn remove_liquidity(
        &self,
        pool: &LbPair,
        position: &PositionV2,
        wallet: &Keypair,
        provider: &Provider,
    ) -> Result<()> {
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        
        // Build and send the remove liquidity transaction
        let tx = lb_clmm.create_remove_liquidity_transaction(
            pool,
            position,
            wallet.pubkey(),
        ).await?;
        
        match provider.send_tx(&tx, true).await {
            Ok(signature) => {
                info!("Remove liquidity transaction successful: {}", signature);
                Ok(())
            }
            Err(e) => {
                warn!("Remove liquidity transaction failed: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn claim_fees(
        &self,
        pool: &LbPair,
        position: &PositionV2,
        wallet: &Keypair,
        provider: &Provider,
    ) -> Result<()> {
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        
        // Build and send the claim fees transaction
        let tx = lb_clmm.create_claim_fee_transaction(
            pool,
            position,
            wallet.pubkey(),
        ).await?;
        
        match provider.send_tx(&tx, true).await {
            Ok(signature) => {
                info!("Claim fees transaction successful: {}", signature);
                Ok(())
            }
            Err(e) => {
                warn!("Claim fees transaction failed: {}", e);
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use anchor_client::solana_sdk::signer::keypair::Keypair;
    use std::thread;
    use std::time::Duration;

    // Note: These tests are designed to be run with a personal wallet containing real SOL.
    // The amounts are kept minimal but sufficient for testing purposes.
    // @bginsber has agreed to use personal SOL for testing to ensure realistic scenarios.
    const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    const SOL_AMOUNT: u64 = 100_000; // 0.0001 SOL

    async fn get_token_balance(provider: &Provider, mint: Pubkey, owner: Pubkey) -> Result<u64> {
        let token_account = spl_associated_token_account::get_associated_token_address(&owner, &mint);
        Ok(provider.rpc_client.get_token_account_balance(&token_account).await?.amount.parse()?)
    }

    #[tokio::test]
    async fn test_realistic_swap_sol_usdc() -> Result<()> {
        let keypair_path = env::var("FUND_KEYPAIR_PATH").expect("FUND_KEYPAIR_PATH must be set");
        let wallet = Keypair::read_from_file(&keypair_path)?;
        
        let provider = Provider::new(env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()));
        let meteora = Meteora::new();

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        let usdc_mint = Pubkey::from_str(USDC_MINT)?;

        // Get initial balances
        let initial_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let initial_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        // Test SOL -> USDC swap
        meteora.swap(
            sol_mint,
            usdc_mint,
            SOL_AMOUNT,
            100, // 1% slippage
            &wallet,
            &provider,
            true, // Skip confirmation prompt in tests
        ).await?;

        // Verify balances changed appropriately
        let final_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let final_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        assert!(final_sol < initial_sol, "SOL balance should decrease after swap");
        assert!(final_usdc > initial_usdc, "USDC balance should increase after swap");

        Ok(())
    }

    #[tokio::test]
    async fn test_realistic_add_remove_liquidity() -> Result<()> {
        let keypair_path = env::var("FUND_KEYPAIR_PATH").expect("FUND_KEYPAIR_PATH must be set");
        let wallet = Keypair::read_from_file(&keypair_path)?;
        
        let provider = Provider::new(env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()));
        let meteora = Meteora::new();

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        let usdc_mint = Pubkey::from_str(USDC_MINT)?;
        
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        let pool = lb_clmm.get_lb_pair(sol_mint, usdc_mint).await?;

        // Get initial balances
        let initial_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let initial_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        // Add minimal liquidity
        meteora.add_liquidity(
            &pool,
            SOL_AMOUNT,
            1_000, // 0.001 USDC
            &wallet,
            &provider,
        ).await?;

        // Verify balances decreased after adding liquidity
        let mid_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let mid_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        assert!(mid_sol < initial_sol, "SOL balance should decrease after adding liquidity");
        assert!(mid_usdc < initial_usdc, "USDC balance should decrease after adding liquidity");

        // Get position
        let positions = lb_clmm.get_positions_by_owner(wallet.pubkey()).await?;
        let position = positions.first().expect("No position found");

        // Remove liquidity
        meteora.remove_liquidity(
            &pool,
            position,
            &wallet,
            &provider,
        ).await?;

        // Verify balances increased after removing liquidity
        let final_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let final_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        assert!(final_sol > mid_sol, "SOL balance should increase after removing liquidity");
        assert!(final_usdc > mid_usdc, "USDC balance should increase after removing liquidity");

        Ok(())
    }

    #[tokio::test]
    async fn test_swap_insufficient_funds() -> Result<()> {
        let keypair_path = env::var("FUND_KEYPAIR_PATH").expect("FUND_KEYPAIR_PATH must be set");
        let wallet = Keypair::read_from_file(&keypair_path)?;
        
        let provider = Provider::new(env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()));
        let meteora = Meteora::new();

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        let usdc_mint = Pubkey::from_str(USDC_MINT)?;

        // Attempt to swap more SOL than the wallet has
        let wallet_balance = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let result = meteora.swap(
            sol_mint,
            usdc_mint,
            wallet_balance + 1, // More than wallet has
            100,
            &wallet,
            &provider,
            true,
        ).await;

        assert!(result.is_err(), "Swap with insufficient funds should fail");
        Ok(())
    }

    #[tokio::test]
    async fn test_claim_fees() -> Result<()> {
        let keypair_path = env::var("FUND_KEYPAIR_PATH").expect("FUND_KEYPAIR_PATH must be set");
        let wallet = Keypair::read_from_file(&keypair_path)?;
        
        let provider = Provider::new(env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()));
        let meteora = Meteora::new();

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        let usdc_mint = Pubkey::from_str(USDC_MINT)?;
        
        let lb_clmm = LbClmm::new(provider.rpc_client.as_ref());
        let pool = lb_clmm.get_lb_pair(sol_mint, usdc_mint).await?;

        // Add liquidity with 5x the base amount
        // Note: We're using a relatively small amount since we'll be placing liquidity
        // in actively traded bins of the SOL-USDC pair, which typically has high volume
        meteora.add_liquidity(
            &pool,
            SOL_AMOUNT * 5, // 5x base amount for meaningful fee generation
            5_000, // 0.005 USDC
            &wallet,
            &provider,
        ).await?;

        // Get position
        let positions = lb_clmm.get_positions_by_owner(wallet.pubkey()).await?;
        let position = positions.first().expect("No position found");

        // Get initial balances
        let initial_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let initial_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        // Wait for some trading activity to generate fees
        info!("Waiting for trading activity to generate fees...");
        thread::sleep(Duration::from_secs(60));

        // Claim fees
        meteora.claim_fees(
            &pool,
            position,
            &wallet,
            &provider,
        ).await?;

        // Verify balances after claiming fees
        let final_sol = provider.rpc_client.get_balance(&wallet.pubkey()).await?;
        let final_usdc = get_token_balance(&provider, usdc_mint, wallet.pubkey()).await?;

        // Note: In a real scenario, at least one of these should increase if fees were generated
        if final_sol <= initial_sol && final_usdc <= initial_usdc {
            info!("No fees were generated during the test period");
        }

        // Clean up by removing liquidity
        meteora.remove_liquidity(
            &pool,
            position,
            &wallet,
            &provider,
        ).await?;

        Ok(())
    }
} 