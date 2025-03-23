use crate::handler::TokenSwapHandler;
use carbon_core::{
    deserialize::ArrangeAccounts, error::CarbonResult,
    instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_meteora_dlmm_decoder::instructions::{
    swap::Swap, MeteoraDlmmInstruction,
};
use std::{collections::HashSet, sync::Arc};

pub struct MeteoraDlmmInstructionProcessor {
    swap_handler: Arc<TokenSwapHandler>,
}

impl MeteoraDlmmInstructionProcessor {
    pub fn new(swap_handler: Arc<TokenSwapHandler>) -> Self {
        Self { swap_handler }
    }
}

#[async_trait::async_trait]
impl Processor for MeteoraDlmmInstructionProcessor {
    type InputType = InstructionProcessorInputType<MeteoraDlmmInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        self.swap_handler.metrics.increment_meteora_dlmm_swaps();
        let (meta, instruction, nested_instructions) = data;
        if let MeteoraDlmmInstruction::Swap(_) = &instruction.data {
            let accounts = Swap::arrange_accounts(&instruction.accounts);
            if let Some(accounts) = accounts {
                let vaults: HashSet<String> = HashSet::from([
                    accounts.reserve_x.to_string(),
                    accounts.reserve_y.to_string(),
                ]);
                self.swap_handler.spawn_swap_processor(
                    &vaults,
                    None,
                    &meta,
                    &nested_instructions,
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod meteora_dlmm_tests {
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
    use carbon_meteora_dlmm_decoder::MeteoraDlmmDecoder;

    async fn test_with_dlmm_decoder(
        tx_hash: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> (
        NestedInstruction,
        Option<DecodedInstruction<MeteoraDlmmInstruction>>,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    ) {
        let (nested_instruction, transaction_update, transaction_metadata) =
            get_nested_instruction(tx_hash, outer_index, inner_index)
                .await
                .expect("Failed to get nested instruction");
        let decoder = MeteoraDlmmDecoder;
        let instruction =
            decoder.decode_instruction(&nested_instruction.instruction);
        (
            nested_instruction,
            instruction,
            transaction_update,
            transaction_metadata,
        )
    }

    /// https://solscan.io/tx/3m4LERWUekW7im8rgu8QgpSJA8a9yEYL3gDvorbd5YpkXarrL3PGoVmyFyQzd1Pw9oZiQy2LPUjaG8Xr4p433kwn
    /// #3.6 - Meteora DLMM Program: swap
    #[tokio::test]
    async fn test_swap_base_output_processor() {
        let signature = "3m4LERWUekW7im8rgu8QgpSJA8a9yEYL3gDvorbd5YpkXarrL3PGoVmyFyQzd1Pw9oZiQy2LPUjaG8Xr4p433kwn";
        let outer_index = 2;
        let inner_index = Some(3);
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_dlmm_decoder(signature, outer_index, inner_index).await;
        let instruction = instruction.expect("Instruction is not some");
        let token_swap_handler = get_token_swap_handler().await;

        let inner_instructions = nested_instruction.inner_instructions.clone();
        let transfers = get_inner_token_transfers(
            &transaction_metadata,
            &inner_instructions,
        )
        .expect("Failed to get inner token transfers");
        assert_eq!(transfers.len(), 2);

        assert_eq!(
            transfers[0],
            TokenTransferDetails {
                amount: 24000000000,
                ui_amount: 24000.0,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "6U91aKa8pmMxkJwBCfPTmUEfZi6dHe7DcFq2ALvB2tbB"
                    .to_string(),
                destination: "CMVrNeYhZnqdbZfQuijgcNvCfvTJN2WKvKSnt2q3HT6N"
                    .to_string(),
                mint: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump"
                    .to_string(),
                source: "89YMNsMDmHeMhT3BiDTcryRuxWSn24B31Gf5H9N2Z8Zu"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                amount: 65256388526,
                ui_amount: 65.256388526,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "6wJ7W3oHj7ex6MVFp2o26NSof3aey7U8Brs8E371WCXA"
                    .to_string(),
                destination: "7x4VcEX8aLd3kFsNWULTp1qFgVtDwyWSxpTGQkoMM6XX"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "5EfbkfLpaz9mHeTN6FnhtN8DTdMGZDRURYcsQ1f1Utg6"
                    .to_string(),
            }
        );

        let mut processor =
            MeteoraDlmmInstructionProcessor::new(token_swap_handler);
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
    }
}
