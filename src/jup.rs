use std::str::FromStr;

use jupiter_swap_api_client::{
    quote::{QuoteRequest, SwapMode},
    swap::SwapRequest,
    transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use log::{error, info};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::VersionedTransaction,
};

use crate::{constants, raydium::SwapArgs, util, Provider};

pub struct Jupiter {
    client: JupiterSwapApiClient,
}

impl Default for Jupiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Jupiter {
    pub fn new() -> Jupiter {
        Jupiter {
            client: JupiterSwapApiClient::new(
                "https://quote-api.jup.ag/v6".to_string(),
            ),
        }
    }

    pub async fn swap_entire_balance(
        &self,
        input_mint: String,
        output_mint: String,
        signer: Keypair,
        provider: Provider,
        confirmed: bool,
        slippage: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spl_token_balance = provider
            .get_spl_balance(&signer.pubkey(), &Pubkey::from_str(&input_mint)?)
            .await?;
        self.swap(SwapArgs {
            input_token_mint: Pubkey::from_str(&input_mint)?,
            output_token_mint: Pubkey::from_str(&output_mint)?,
            amount: spl_token_balance,
            wallet: signer,
            provider,
            confirmed,
            slippage: slippage as u64,
            amm_pool: Pubkey::default(), // unused
            no_sanity: false,            // unused
        })
        .await
    }

    pub async fn swap(
        &self,
        swap_args: SwapArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let SwapArgs {
            input_token_mint: input_mint,
            output_token_mint: output_mint,
            amount,
            wallet: signer,
            provider,
            confirmed,
            slippage,
            amm_pool: _,
            no_sanity: _,
        } = swap_args;
        let start = std::time::Instant::now();
        if !confirmed {
            info!(
                "Initializing swap of {} of {} for {} by {}, slippage: {}%",
                {
                    if input_mint.eq(&constants::SOLANA_PROGRAM_ID) {
                        util::lamports_to_sol(amount)
                    } else {
                        amount as f64
                    }
                },
                input_mint,
                output_mint,
                signer.pubkey(),
                slippage as f32 / 100.
            );
            if !dialoguer::Confirm::new()
                .with_prompt("Go for it?")
                .interact()?
            {
                return Ok(());
            };
        }
        let quote_response = self
            .client
            .quote(&QuoteRequest {
                input_mint,
                output_mint,
                amount,
                slippage_bps: slippage as u16,
                swap_mode: Some(SwapMode::ExactIn),
                ..QuoteRequest::default()
            })
            .await?;
        let swap_response = self
            .client
            .swap(&SwapRequest {
                user_public_key: signer.pubkey(),
                quote_response,
                config: TransactionConfig::default(),
            })
            .await?;
        let raw_tx: VersionedTransaction =
            bincode::deserialize(&swap_response.swap_transaction).unwrap();
        let signed_tx =
            VersionedTransaction::try_new(raw_tx.message, &[&signer])?;

        info!("Built tx in {:?}", start.elapsed());
        match provider.send_tx(&signed_tx, true).await {
            Ok(signature) => {
                info!("Transaction {} successful", signature);
                Ok(())
            }
            Err(e) => {
                error!("Transaction failed: {}", e);
                Err(e)
            }
        }
    }
}
