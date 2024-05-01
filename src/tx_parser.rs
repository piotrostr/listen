use core::panic;
use std::collections::HashMap;

use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction, UiTransactionTokenBalance,
};

use crate::{constants, util, Swap};

pub fn parse_mint(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<String, Box<dyn std::error::Error>> {
    let instructions = self::parse_instructions(tx)?;
    for instruction in instructions {
        match instruction {
            UiInstruction::Parsed(ix) => match ix {
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
            },
            _ => (),
        }
    }
    return Err("Mint not found in tx".into());
}

pub fn parse_tmp_account(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<String, Box<dyn std::error::Error>> {
    let instructions = self::parse_instructions(tx)?;
    let mut tmp_account = String::new();
    for instruction in instructions {
        match instruction {
            UiInstruction::Parsed(ix) => match ix {
                UiParsedInstruction::Parsed(ix) => {
                    if ix.program == "spl-token"
                        && ix.parsed["type"] == "closeAccount"
                    {
                        tmp_account = ix.parsed["info"]["account"].to_string();
                    }
                }
                UiParsedInstruction::PartiallyDecoded(_) => {}
            },
            _ => (),
        }
    }

    if tmp_account == "" {
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
                    match ix {
                        UiInstruction::Parsed(UiParsedInstruction::Parsed(
                            parsed_ix,
                        )) => {
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
                                        constants::SOLANA_PROGRAM_ID
                                            .to_string();
                                    swap.quote_amount = amount;
                                };
                            }
                        }
                        _ => (),
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
