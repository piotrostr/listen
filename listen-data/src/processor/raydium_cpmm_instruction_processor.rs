use crate::handler::{token_swap_handler::Dex, TokenSwapHandler};
use carbon_core::{
    deserialize::ArrangeAccounts, error::CarbonResult,
    instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_cpmm_decoder::instructions::{
    swap_base_input::SwapBaseInput, swap_base_output::SwapBaseOutput,
    RaydiumCpmmInstruction,
};
use std::{collections::HashSet, sync::Arc};

pub struct RaydiumCpmmInstructionProcessor {
    swap_handler: Arc<TokenSwapHandler>,
}

impl RaydiumCpmmInstructionProcessor {
    pub fn new(swap_handler: Arc<TokenSwapHandler>) -> Self {
        Self { swap_handler }
    }
}

#[async_trait::async_trait]
impl Processor for RaydiumCpmmInstructionProcessor {
    type InputType = InstructionProcessorInputType<RaydiumCpmmInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        self.swap_handler.metrics.increment_raydium_cpmm_swaps();
        let (meta, instruction, nested_instructions) = data;
        match &instruction.data {
            RaydiumCpmmInstruction::SwapBaseInput(_) => {
                let accounts =
                    SwapBaseInput::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.input_token_account.to_string(),
                        accounts.output_token_account.to_string(),
                    ]);
                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        None,
                        &meta,
                        &nested_instructions,
                        Dex::RaydiumCpmm,
                    );
                }
            }
            RaydiumCpmmInstruction::SwapBaseOutput(_) => {
                let accounts =
                    SwapBaseOutput::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.input_token_account.to_string(),
                        accounts.output_token_account.to_string(),
                    ]);
                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        None,
                        &meta,
                        &nested_instructions,
                        Dex::RaydiumCpmm,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod cpmm_tests {
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
    use carbon_raydium_cpmm_decoder::RaydiumCpmmDecoder;

    async fn test_with_clmm_decoder(
        tx_hash: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> (
        NestedInstruction,
        Option<DecodedInstruction<RaydiumCpmmInstruction>>,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    ) {
        let (nested_instruction, transaction_update, transaction_metadata) =
            get_nested_instruction(tx_hash, outer_index, inner_index)
                .await
                .expect("Failed to get nested instruction");
        let decoder = RaydiumCpmmDecoder;
        let instruction =
            decoder.decode_instruction(&nested_instruction.instruction);
        (
            nested_instruction,
            instruction,
            transaction_update,
            transaction_metadata,
        )
    }

    /// https://solscan.io/tx/7Gr8Wtzd3T6L4YwGnH98eKbNnitbTPjTfiYy8pdnebA6sHZRYJztw76ruPGSjDe8DciEqVtyuA8VqGjxiJ7UVr5
    /// #5 - Raydium CPMM: swapBaseInput
    #[tokio::test]
    async fn test_swap_base_input_processor() {
        let tx_hash = "7Gr8Wtzd3T6L4YwGnH98eKbNnitbTPjTfiYy8pdnebA6sHZRYJztw76ruPGSjDe8DciEqVtyuA8VqGjxiJ7UVr5";
        let outer_index = 4;
        let inner_index = None;
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_clmm_decoder(tx_hash, outer_index, inner_index).await;
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
                amount: 301710000,
                ui_amount: 0.30171,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "Br7CVfr3536FC66QkAU8TM1BrdupFqWt3zhknze7ubz"
                    .to_string(),
                destination: "DvPZP2ZXpAP1CCoJk4LmTet97YWJ8nkjNSSFyo4dzAvF"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "4LbQZSQvHix6sNTo4VCLM2gLTBe32JkQRJFuWGCGp7fi"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                amount: 7214400496961,
                ui_amount: 7214.400496961,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL"
                    .to_string(),
                destination: "DDG6sRgMkUGdWsWDenCkSoYy5DtGmm4bLVR2XUcTTRPf"
                    .to_string(),
                mint: "866Sh46xjH7cW7aW18tBUmGm3xh6EzGTk1Li7YbbmqJr"
                    .to_string(),
                source: "HxT2zqXpWcoWbB5KxkDNydm659Ndxn5mvkza1C3js2tu"
                    .to_string(),
            }
        );

        let mut processor =
            RaydiumCpmmInstructionProcessor::new(token_swap_handler);
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

    /// https://solscan.io/tx/4fzSVEeGUZGGooNYEPoEjH1wSXwRh4X5yv2SAWQ723SwncCVkgQNdC5qEMFAyDYzbAEp5xHYnqhMZLDS78xLSHt4
    /// #3 - Raydium CPMM: swapBaseOutput
    #[tokio::test]
    async fn test_swap_base_output_processor() {
        let tx_hash = "4fzSVEeGUZGGooNYEPoEjH1wSXwRh4X5yv2SAWQ723SwncCVkgQNdC5qEMFAyDYzbAEp5xHYnqhMZLDS78xLSHt4";
        let outer_index = 2;
        let inner_index = None;
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_clmm_decoder(tx_hash, outer_index, inner_index).await;
        let instruction = instruction.expect("Instruction is not some");
        let token_swap_processor = get_token_swap_handler().await;

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
                amount: 322356054,
                ui_amount: 0.322356054,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "GCvHhEUYQwTJ8jyf8Lc4bv8jBXyZU4LMMsZCobwEPzvM"
                    .to_string(),
                destination: "5hVU9W7s2g2VRjQR4Hzz5rchwnRP7MZN7Fm1AfSWd3bA"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "BUuuCwv3vDLxrhsiy4VZEx7oQjk6nE1Xn1nK7KsPneTE"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                amount: 49772609000,
                ui_amount: 49772609.0,
                decimals: 3,
                program_id: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                    .to_string(),
                authority: "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL"
                    .to_string(),
                destination: "6wMp1WC5cfmH5Q9xTRW2atJaDetXCRGv8Kt9Z7LsxQr7"
                    .to_string(),
                mint: "pi1RgmNaLQsNEyEAsrEjgmemojPwitwDAXc3zgseWWF".to_string(),
                source: "6k3qWpmArZS8S1MmRiXUhWceVAnMWJnn3sDRUxcpcC35"
                    .to_string(),
            }
        );

        let mut processor =
            RaydiumCpmmInstructionProcessor::new(token_swap_processor.clone());
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
