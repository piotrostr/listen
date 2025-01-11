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