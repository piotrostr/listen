use crate::handler::TokenSwapHandler;
use carbon_core::{
    deserialize::ArrangeAccounts, error::CarbonResult,
    instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_orca_whirlpool_decoder::instructions::{
    swap::Swap, OrcaWhirlpoolInstruction,
};
use std::{collections::HashSet, sync::Arc};

pub struct OcraWhirlpoolInstructionProcessor {
    swap_handler: Arc<TokenSwapHandler>,
}

impl OcraWhirlpoolInstructionProcessor {
    pub fn new(swap_handler: Arc<TokenSwapHandler>) -> Self {
        Self { swap_handler }
    }
}

#[async_trait::async_trait]
impl Processor for OcraWhirlpoolInstructionProcessor {
    type InputType = InstructionProcessorInputType<OrcaWhirlpoolInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        self.swap_handler.metrics.increment_whirlpools_swaps();
        let (meta, instruction, nested_instructions) = data;
        if let OrcaWhirlpoolInstruction::Swap(_) = &instruction.data {
            let accounts = Swap::arrange_accounts(&instruction.accounts);
            if let Some(accounts) = accounts {
                let vaults: HashSet<String> = HashSet::from([
                    accounts.token_vault_a.to_string(),
                    accounts.token_vault_b.to_string(),
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
mod orca_whirlpool_tests {
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
    use carbon_orca_whirlpool_decoder::OrcaWhirlpoolDecoder;

    async fn test_with_whirlpool_decoder(
        tx_hash: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> (
        NestedInstruction,
        Option<DecodedInstruction<OrcaWhirlpoolInstruction>>,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    ) {
        let (nested_instruction, transaction_update, transaction_metadata) =
            get_nested_instruction(tx_hash, outer_index, inner_index)
                .await
                .expect("Failed to get nested instruction");
        let decoder = OrcaWhirlpoolDecoder;
        let instruction =
            decoder.decode_instruction(&nested_instruction.instruction);
        (
            nested_instruction,
            instruction,
            transaction_update,
            transaction_metadata,
        )
    }

    /// https://solscan.io/tx/3ankeujUXU4EPjcJXFdNrn4nqGVati1KpMntYfTpgGhboxywLVb2oYpG9BStMwGojjvGSfNff4Zar8tPqX9ifJMP
    /// #2 - Whirlpools Program: swap
    #[tokio::test]
    async fn test_swap_base_output_processor() {
        let signature = "3ankeujUXU4EPjcJXFdNrn4nqGVati1KpMntYfTpgGhboxywLVb2oYpG9BStMwGojjvGSfNff4Zar8tPqX9ifJMP";
        let outer_index = 1;
        let inner_index = None;
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_whirlpool_decoder(signature, outer_index, inner_index)
                .await;
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
                amount: 1961878075,
                ui_amount: 1961.878075,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa"
                    .to_string(),
                destination: "79Lv5tG6n74sRJFLjrXxwqBdNmFv8ERYQZ1WiSUbCDU4"
                    .to_string(),
                mint: "61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump"
                    .to_string(),
                source: "3g4yFngFJyQppCFcaD2sbPe4HdLzQiS64MfPSPLK5iN5"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                amount: 1241037050,
                ui_amount: 1.24103705,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "57mP5WoNrg3uiGFUdoeYr2CPUZak1L2ZgFtyFwoT7K6G"
                    .to_string(),
                destination: "CTyFguG69kwYrzk24P3UuBvY1rR5atu9kf2S6XEwAU8X"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "CcwLMXxRLaaf1biHSaXCckQB85xyq3U7GRo3iiqCV74H"
                    .to_string(),
            }
        );

        let mut processor =
            OcraWhirlpoolInstructionProcessor::new(token_swap_handler);
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
