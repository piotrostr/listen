use std::collections::HashMap;

use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction,
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

pub fn parse_swap_from_balances_change(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Vec<HashMap<&str, String>> {
    let mut changes = vec![];
    if let Some(meta) = &tx.transaction.meta {
        // zip pre and post balances
        for (pre, post) in std::iter::zip(
            self::deserialize(&meta.pre_token_balances),
            self::deserialize(&meta.post_token_balances),
        ) {
            if pre.ui_token_amount.ui_amount.is_none()
                || post.ui_token_amount.ui_amount.is_none()
            {
                continue;
            }
            let diff = post.ui_token_amount.ui_amount.unwrap()
                - pre.ui_token_amount.ui_amount.unwrap();
            let mint = pre.mint.to_string();
            let mut owner = self::deserialize(&pre.owner).to_string();
            if owner == constants::RAYDIUM_AUTHORITY_V4_PUBKEY {
                owner = "RAYDIUM_AUTHORITY".to_string();
            }
            let mut m = HashMap::new();
            m.insert("mint", mint.to_string());
            m.insert("owner", owner.to_string());
            m.insert("diff", diff.to_string());
            changes.push(m);
        }
    }
    return changes;
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
