use base64::Engine;
use core::panic;
use log::info;
use std::str::FromStr;

use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use solana_transaction_status::{
    option_serializer::OptionSerializer,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    UiInstruction, UiMessage, UiParsedInstruction,
};
use timed::timed;

use crate::{constants, util, Swap};

#[derive(Debug, Default)]
pub struct NewPool {
    pub amm_pool_id: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub creator: Pubkey,
}

#[timed(duration(printer = "info!"))]
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
                            .ok_or("Failed to get string")?
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

#[timed(duration(printer = "info!"))]
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

#[timed(duration(printer = "info!"))]
pub fn parse_notional(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<u64, Box<dyn std::error::Error>> {
    if let Some(meta) = &tx.transaction.meta {
        let max_sol = std::iter::zip(&meta.pre_balances, &meta.post_balances)
            .map(|(a, b)| (*a as f64 - *b as f64) as u64)
            .max()
            .ok_or("Failed to get max")?;
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

#[timed(duration(printer = "info!"))]
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
                                .to_string()
                        && pool_info.amm_pool_id == Pubkey::default()
                    {
                        pool_info.amm_pool_id = Pubkey::from_str(
                            parsed_ix.parsed["info"]["account"]
                                .as_str()
                                .ok_or("Failed to get string")?,
                        )?;
                    }

                    if parsed_ix.parsed["type"] == "initializeAccount"
                        && parsed_ix.parsed["info"]["owner"]
                            == constants::RAYDIUM_AUTHORITY_V4_PUBKEY
                                .to_string()
                    {
                        let mint = parsed_ix.parsed["info"]["mint"]
                            .as_str()
                            .ok_or("Failed to get string")?;
                        if mint == constants::SOLANA_PROGRAM_ID.to_string() {
                            pool_info.input_mint =
                                constants::SOLANA_PROGRAM_ID;
                        } else {
                            pool_info.output_mint = Pubkey::from_str(mint)?;
                        }
                    }

                    if parsed_ix.program == "system" {
                        if let Some(owner) =
                            parsed_ix.parsed["info"]["source"].as_str()
                        {
                            pool_info.creator = Pubkey::from_str(owner)?;
                        }
                    }
                }
            }
        }
    }
    Ok(pool_info)
}

#[timed(duration(printer = "info!"))]
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
                    if let UiInstruction::Parsed(
                        UiParsedInstruction::Parsed(parsed_ix),
                    ) = ix
                    {
                        if parsed_ix.program == "spl-token"
                            && parsed_ix.parsed["type"] == "transfer"
                        {
                            let amount = parsed_ix.parsed["info"]["amount"]
                                .as_str()
                                .ok_or("Failed to get string")?
                                .parse::<f64>()?;
                            // if the authority is raydium, it is the shitcoin, otherwise SOL
                            if parsed_ix.parsed["info"]["authority"]
                                == constants::RAYDIUM_AUTHORITY_V4_PUBKEY
                                    .to_string()
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

#[timed(duration(printer = "info!"))]
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

/// decode_tx decodes a base64 transaction (send from another txlisten service in Go)
/// this won't be used since the transaction format differs to what is expected
/// (EncodedTransactionWithStatusMeta in Rust)
/// solana SDK is trashy I must say..
pub fn decode_tx(raw_tx: String) -> Result<(), Box<dyn std::error::Error>> {
    let raw_bytes = base64::prelude::BASE64_STANDARD.decode(raw_tx)?;
    let tx: Transaction = bincode::deserialize(&raw_bytes)?;

    println!("{:?}", tx);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_decode_tx() {
        let raw_tx = "AUqzoqR28ec7nh+XuPEaQ8GDEJRtpdlg+kILyL1G2L7kQ836DqwhnP3AvpqaiR6TmcQAGXwITv5vf0kXu3gzkgeAAQAMF847x3GeLV4VO7eN63MSkgp+jLbu8dka4OKEBgV8ak44ib5hiJjILmxsUCBjFgn2Y4f7QVjWdziegObSFvuzPznNlQVsnTFEmlxsOoCriIOCXTyj25c2tlOF7ZzEKjvB26bZiJ4Hp1o/26BYg9csa7uNdsxPX+gRuGhVECd45oLs/YVwGKWUV2zfHZE2qkR1ddcRpoZU2qv56tYLo+pLf/dTU0/nk/TVeZYR3wgCK+eFqp79zzNVkIzse9MiwmTYW79BkWX5JaN29q8gjSENqQ6KzYMz1INh3Dn4T0F9MzYwI4WjpKu1O+YYZBbhsgVU7zh/COag0oI/cp40afjQFBlhUcscPp0g/YRKOoHODeiXX/AHv7/KUbB30tLvBJnYDkCJDBbp4Rc5wIo5MjGjRzeuQLlliyB4Fqg1PomFbG3fBAU+rsk3XIxisOEl64y2KaBE51AzJolm/VGwofTutTUDBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAQan1RcZLFxRIYzJTD1K8X9Y2u4Im6H9ROPb2YoAAAAAS9lJxDYCwz8gd5DtFqNSTKG5l1zxIaKpDP/sffi2is2MlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WUFXsFgPMcX85EpiWC28+deO51lDoISjk7NQNo0iiZMI0hWQJd+mLVEHAODIJNao5niz8Efr3wgqrc/u/TZz5u15/f3z/y6DYV6qShd68DAYic45gTgcm5TTuPJ8CAeYLA0HUagoLaYTBf4pnDe5mOWEcdsRNQNzEPi+EEWmCvbu04oIcnkuTE3nX4rVkwcCBnT6Djny2yHDBDANkf2+Bz70YiUZmfT1BiEModHgTed58H5UTgPBni7NojS7m0KecAYLAAkDQUIPAAAAAAALAAUCQEIPAAwCAAF8AwAAAM47x3GeLV4VO7eN63MSkgp+jLbu8dka4OKEBgV8ak44IAAAAAAAAABGUFJScFpLOWs3Z21hOGlVc3BEd2JTaVlTUkZvVm45SvCRWqQLAAAApQAAAAAAAAAG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQ0EAQ4ADwEBEBUNEQwPAhIDBBMOBQYHFAgVFgAJAQoaAf6421FmAAAAAAB0O6QLAAAAAADaSTtxfQwNAwEAAAEJAA==".to_string();
        super::decode_tx(raw_tx).unwrap();
    }
}
