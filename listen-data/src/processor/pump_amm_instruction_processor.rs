use crate::handler::{token_swap_handler::Dex, TokenSwapHandler};
use carbon_core::{
    deserialize::ArrangeAccounts, error::CarbonResult,
    instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_pump_swap_decoder::instructions::{
    buy::Buy, sell::Sell, PumpSwapInstruction,
};
use std::{collections::HashSet, sync::Arc};

pub struct PumpAmmInstructionProcessor {
    swap_handler: Arc<TokenSwapHandler>,
}

impl PumpAmmInstructionProcessor {
    pub fn new(swap_handler: Arc<TokenSwapHandler>) -> Self {
        Self { swap_handler }
    }
}

#[async_trait::async_trait]
impl Processor for PumpAmmInstructionProcessor {
    type InputType = InstructionProcessorInputType<PumpSwapInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        self.swap_handler.metrics.increment_pump_swaps();
        let (meta, instruction, nested_instructions) = data;

        match &instruction.data {
            PumpSwapInstruction::Buy(_) => {
                let accounts = Buy::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_base_token_account.to_string(),
                        accounts.pool_quote_token_account.to_string(),
                    ]);
                    let fee_adas = HashSet::from([accounts
                        .protocol_fee_recipient_token_account
                        .to_string()]);

                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        Some(&fee_adas),
                        &meta,
                        &nested_instructions,
                        Dex::PumpSwap,
                    );
                }
            }
            PumpSwapInstruction::Sell(_) => {
                let accounts = Sell::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_base_token_account.to_string(),
                        accounts.pool_quote_token_account.to_string(),
                    ]);
                    let fee_adas = HashSet::from([accounts
                        .protocol_fee_recipient_token_account
                        .to_string()]);

                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        Some(&fee_adas),
                        &meta,
                        &nested_instructions,
                        Dex::PumpSwap,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod pump_amm_tests {
    use super::*;
    use crate::{
        diffs::TokenTransferDetails,
        handler::token_swap_handler::test_swaps::{
            get_inner_token_transfers, get_nested_instruction,
            get_token_swap_handler,
        },
    };
    use carbon_core::{
        datasource::TransactionUpdate,
        instruction::{
            DecodedInstruction, InstructionDecoder, NestedInstruction,
        },
        transaction::TransactionMetadata,
    };
    use carbon_pump_swap_decoder::PumpSwapDecoder;

    async fn test_with_pump_decoder(
        tx_hash: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> (
        NestedInstruction,
        Option<DecodedInstruction<PumpSwapInstruction>>,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    ) {
        let (nested_instruction, transaction_update, transaction_metadata) =
            get_nested_instruction(tx_hash, outer_index, inner_index)
                .await
                .expect("Failed to get nested instruction");
        let decoder = PumpSwapDecoder;
        let instruction =
            decoder.decode_instruction(&nested_instruction.instruction);
        (
            nested_instruction,
            instruction,
            transaction_update,
            transaction_metadata,
        )
    }

    /// https://solscan.io/tx/3G7iGWpatj5vjPRmsxRsYh3N6B1WkiBX77u8yizPVcGZkqytdT6UYeCfsHan816sRH3jYpG45FRL3GLywud7CpbT
    /// Swap 2,523 DWH for 0.007229486 $0.9411 WSOL On Pump.fun AMM
    #[tokio::test]
    async fn test_sell_processor() {
        let signature = "3G7iGWpatj5vjPRmsxRsYh3N6B1WkiBX77u8yizPVcGZkqytdT6UYeCfsHan816sRH3jYpG45FRL3GLywud7CpbT";
        let outer_index = 0;
        let inner_index = None;
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_pump_decoder(signature, outer_index, inner_index).await;
        let instruction = instruction.expect("Instruction is not some");
        let token_swap_handler = get_token_swap_handler().await;

        let inner_instructions = nested_instruction.inner_instructions.clone();
        let transfers = get_inner_token_transfers(
            &transaction_metadata,
            &inner_instructions,
        )
        .expect("Failed to get inner token transfers");
        assert_eq!(transfers.len(), 3);

        assert_eq!(
            transfers[0],
            TokenTransferDetails {
                amount: 2523000000,
                ui_amount: 2523.0,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "G2gUder2Y934cm8ufSQxjbhjrfJsiBBAox1jgLqEDx75"
                    .to_string(),
                destination: "GHs3Cs9J6NoX79Nr2KvR1Nnzm82R34Jmqh1A8Bb84zgc"
                    .to_string(),
                mint: "2WZuixz3wohXbib7Ze2gRjVeGeESiMw9hsizDwbjM4YK"
                    .to_string(),
                source: "yAcYcbC9Qr9SBpeG9SbT1zAEFwHd8j6EFFWomjQjVtn"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                amount: 7229486,
                ui_amount: 0.007229486,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "ezWtvReswwZaEBThCnW23qtH5uANic2akGY7yh7vZR9"
                    .to_string(),
                destination: "6qxghyVLU7sVYhQn6JKziDqb2VMPuDS6Q6rGngnkXdxx"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "4UKfPxrJGEXggv637xCbzethVUGtkv6vay5zCjDSg1Yb"
                    .to_string(),
            }
        );

        // fee
        assert_eq!(
            transfers[2],
            TokenTransferDetails {
                amount: 3624,
                ui_amount: 0.000003624,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "ezWtvReswwZaEBThCnW23qtH5uANic2akGY7yh7vZR9"
                    .to_string(),
                destination: "Bvtgim23rfocUzxVX9j9QFxTbBnH8JZxnaGLCEkXvjKS"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "4UKfPxrJGEXggv637xCbzethVUGtkv6vay5zCjDSg1Yb"
                    .to_string(),
            }
        );

        let mut processor =
            PumpAmmInstructionProcessor::new(token_swap_handler.clone());
        processor
            .process(
                (
                    nested_instruction.metadata.clone(),
                    instruction,
                    nested_instruction.inner_instructions.clone(),
                ),
                Arc::new(MetricsCollection::new(vec![])),
            )
            .await
            .expect("Failed to process instruction");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
