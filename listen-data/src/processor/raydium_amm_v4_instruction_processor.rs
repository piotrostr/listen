use crate::handler::{token_swap_handler::Dex, TokenSwapHandler};
use carbon_core::{
    deserialize::ArrangeAccounts, error::CarbonResult,
    instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::{
    swap_base_in::SwapBaseIn, swap_base_out::SwapBaseOut,
    RaydiumAmmV4Instruction,
};
use std::{collections::HashSet, sync::Arc};

pub struct RaydiumAmmV4InstructionProcessor {
    pub swap_handler: Arc<TokenSwapHandler>,
}

impl RaydiumAmmV4InstructionProcessor {
    pub fn new(swap_handler: Arc<TokenSwapHandler>) -> Self {
        Self { swap_handler }
    }
}

#[async_trait::async_trait]
impl Processor for RaydiumAmmV4InstructionProcessor {
    type InputType = InstructionProcessorInputType<RaydiumAmmV4Instruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        self.swap_handler.metrics.increment_raydium_amm_v4_swaps();
        let (meta, instruction, nested_instructions) = data;
        match &instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(_) => {
                let accounts =
                    SwapBaseIn::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_coin_token_account.to_string(),
                        accounts.pool_pc_token_account.to_string(),
                    ]);
                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        None,
                        &meta,
                        &nested_instructions,
                        Dex::RaydiumAmmV4,
                    );
                }
            }
            RaydiumAmmV4Instruction::SwapBaseOut(_) => {
                let accounts =
                    SwapBaseOut::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_coin_token_account.to_string(),
                        accounts.pool_pc_token_account.to_string(),
                    ]);
                    self.swap_handler.spawn_swap_processor(
                        &vaults,
                        None,
                        &meta,
                        &nested_instructions,
                        Dex::RaydiumAmmV4,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod amm_v4_tests {
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
    use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;

    async fn test_with_amm_v4_decoder(
        tx_hash: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> (
        NestedInstruction,
        Option<DecodedInstruction<RaydiumAmmV4Instruction>>,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    ) {
        let (nested_instruction, transaction_update, transaction_metadata) =
            get_nested_instruction(tx_hash, outer_index, inner_index)
                .await
                .expect("Failed to get nested instruction");
        let decoder = RaydiumAmmV4Decoder;
        let instruction =
            decoder.decode_instruction(&nested_instruction.instruction);
        (
            nested_instruction,
            instruction,
            transaction_update,
            transaction_metadata,
        )
    }

    #[tokio::test]
    async fn test_spawn_swap_processor() {
        let tx_hash = "31pB39KowUTdDSjXhzCYi7QxVSWSM4ZijaSWAkCduWUUR6GuGrWwVBbcXLLdJnVLrWbQaV7YFL2SigBXRatGfnji";
        let outer_index = 3;
        let inner_index = Some(1);
        let (nested_instruction, instruction, _, transaction_metadata) =
            test_with_amm_v4_decoder(tx_hash, outer_index, inner_index).await;
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
                mint: "AsyfR3e5JcPqWot4H5MMhQUm7DZ4zwQrcp2zbB7vpump"
                    .to_string(),
                amount: 279274681533,
                ui_amount: 279274.681533,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                source: "3oV3EFEp6GUTt8cn3swj1oQXhmeuRyKv9cEzpSVZga5K"
                    .to_string(),
                destination: "HqDtzxBsHHhmTHbzmUk5aJkAZE8iGf6KKeeYrh4mVCc3"
                    .to_string(),
                authority: "6LXutJvKUw8Q5ue2gCgKHQdAN4suWW8awzFVC6XCguFx"
                    .to_string(),
            }
        );

        assert_eq!(
            transfers[1],
            TokenTransferDetails {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                amount: 8569783440,
                ui_amount: 8.56978344,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                source: "6M2KAV658rer6g2L7tAAQtXK7f1GmrbG7ycW14gHdK5U"
                    .to_string(),
                destination: "BuqEDKUwyAotZuK37V4JYEykZVKY8qo1zKbpfU9gkJMo"
                    .to_string(),
                authority: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                    .to_string(),
            }
        );

        let mut processor =
            RaydiumAmmV4InstructionProcessor::new(token_swap_handler);
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
