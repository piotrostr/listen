use core::panic;
use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction,
};

use crate::{constants, util, Swap};

#[derive(Debug, Default)]
pub struct NewPool {
    pub amm_pool_id: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
}

pub fn parse_mint(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<String, Box<dyn std::error::Error>> {
    let instructions = self::parse_instructions(tx)?;
    for instruction in instructions {
        if let UiInstruction::Parsed(ix) = instruction {
            match ix {
                UiParsedInstruction::Parsed(ix) => {
                    if ix.program == "spl-associated-token-account" {
                        // TODO this might panic, might be handled more gracefully
                        let mint = ix.parsed["info"]["mint"]
                            .as_str()
                            .unwrap()
                            .to_string();
                        return Ok(mint);
                    }
                }
                UiParsedInstruction::PartiallyDecoded(_) => (),
            }
        }
    }
    Err("Mint not found in tx".into())
}

pub fn parse_tmp_account(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<String, Box<dyn std::error::Error>> {
    let instructions = self::parse_instructions(tx)?;
    let mut tmp_account = String::new();
    for instruction in instructions {
        if let UiInstruction::Parsed(ix) = instruction {
            match ix {
                UiParsedInstruction::Parsed(ix) => {
                    if ix.program == "spl-token"
                        && ix.parsed["type"] == "closeAccount"
                    {
                        tmp_account = ix.parsed["info"]["account"].to_string();
                    }
                }
                UiParsedInstruction::PartiallyDecoded(_) => {}
            }
        }
    }

    if tmp_account.is_empty() {
        return Err("Temp account not found".into());
    }

    Ok(tmp_account)
}

pub fn parse_signer() -> Result<String, Box<dyn std::error::Error>> {
    // TODO
    Err("Not implemented".into())
}

pub fn parse_notional(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<u64, Box<dyn std::error::Error>> {
    if let Some(meta) = &tx.transaction.meta {
        let max_sol = std::iter::zip(&meta.pre_balances, &meta.post_balances)
            .map(|(a, b)| (*a as f64 - *b as f64) as u64)
            .max()
            .unwrap();
        return Ok(max_sol);
    }
    Err("could not parse notional".into())
}

pub fn deserialize<T: Clone>(item: &OptionSerializer<T>) -> T {
    match item {
        OptionSerializer::Some(val) => val.clone(),
        _ => panic!("Deserialization failed"),
    }
}

pub fn parse_new_pool(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<NewPool, Box<dyn std::error::Error>> {
    let mut pool_info = NewPool::default();
    if let Some(meta) = &tx.transaction.meta {
        for ixs in self::deserialize(&meta.inner_instructions) {
            for ix in ixs.instructions.iter().rev() {
                if let UiInstruction::Parsed(UiParsedInstruction::Parsed(
                    parsed_ix,
                )) = ix
                {
                    if parsed_ix.parsed["type"] == "assign"
                        && parsed_ix.parsed["info"]["owner"]
                            == constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                        && pool_info.amm_pool_id == Pubkey::default()
                    {
                        pool_info.amm_pool_id = Pubkey::from_str(
                            parsed_ix.parsed["info"]["account"]
                                .as_str()
                                .unwrap(),
                        )
                        .unwrap();
                    }

                    if parsed_ix.parsed["type"] == "initializeAccount"
                        && parsed_ix.parsed["info"]["owner"]
                            == constants::RAYDIUM_AUTHORITY_V4_PUBKEY
                    {
                        let mint =
                            parsed_ix.parsed["info"]["mint"].as_str().unwrap();
                        if mint == constants::SOLANA_PROGRAM_ID {
                            pool_info.input_mint =
                                Pubkey::from_str(constants::SOLANA_PROGRAM_ID)
                                    .unwrap();
                        } else {
                            pool_info.output_mint =
                                Pubkey::from_str(mint).unwrap();
                        }
                    }
                }
            }
        }
    }
    Ok(pool_info)
}

pub fn parse_swap(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<Swap, Box<dyn std::error::Error>> {
    let mut swap = Swap::default();
    if let Some(meta) = &tx.transaction.meta {
        let all_ixs = self::deserialize(&meta.inner_instructions);
        for ixs in all_ixs {
            // might also be identified based on static index 5 but
            // that would be even more brittle than this
            if ixs.instructions.len() == 2 {
                for ix in ixs.instructions {
                    if let UiInstruction::Parsed(UiParsedInstruction::Parsed(
                        parsed_ix,
                    )) = ix
                    {
                        if parsed_ix.program == "spl-token"
                            && parsed_ix.parsed["type"] == "transfer"
                        {
                            let amount = parsed_ix.parsed["info"]["amount"]
                                .as_str()
                                .unwrap()
                                .parse::<f64>()
                                .unwrap();
                            // if the authority is raydium, it is the shitcoin, otherwise SOL
                            if parsed_ix.parsed["info"]["authority"]
                                == constants::RAYDIUM_AUTHORITY_V4_PUBKEY
                            {
                                // shitcoin == base quote, like POOP/SOL
                                swap.base_mint = self::parse_mint(tx)?;
                                swap.base_amount = amount;
                            } else {
                                // TODO not sure how to support non-SOL
                                // swaps yet also does not return the
                                // mint token properly
                                swap.quote_mint =
                                    constants::SOLANA_PROGRAM_ID.to_string();
                                swap.quote_amount = amount;
                            };
                        }
                    }
                }
                swap.sol_amount_ui =
                    util::lamports_to_sol(swap.quote_amount as u64);
            }
        }
    }

    Ok(swap)
}

pub fn parse_instructions(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<Vec<UiInstruction>, Box<dyn std::error::Error>> {
    match &tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Parsed(msg) => Ok(msg.instructions.clone()),
            UiMessage::Raw(_) => Err("Raw message not supported".into()),
        },
        _ => Err("Only EncodedTransaction::Json txs are supported".into()),
    }
}
