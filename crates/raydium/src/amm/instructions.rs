use crate::amm;
use amm::openbook;
use amm::utils::AmmKeys;
use anyhow::Result;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn initialize_amm_pool(
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    create_fee_detination: &Pubkey,
    user_owner: &Pubkey,
    user_coin: &Pubkey,
    user_pc: &Pubkey,
    user_lp: &Pubkey,
    open_time: u64, // default is 0, or set a future time on the chain can start swap
    pc_amount: u64, // transfer pc asset to the pool pc vault as pool init vault
    coin_amount: u64, // transfer coin asset to the pool coin vault as pool init vault
) -> Result<Instruction> {
    let amm_pool_init_instruction = raydium_amm::instruction::initialize2(
        &amm_program,
        &amm_keys.amm_pool,
        &amm_keys.amm_authority,
        &amm_keys.amm_open_order,
        &amm_keys.amm_lp_mint,
        &amm_keys.amm_coin_mint,
        &amm_keys.amm_pc_mint,
        &amm_keys.amm_coin_vault,
        &amm_keys.amm_pc_vault,
        &amm_keys.amm_target,
        &Pubkey::find_program_address(
            &[&raydium_amm::processor::AMM_CONFIG_SEED],
            &amm_program,
        )
        .0,
        create_fee_detination,
        &amm_keys.market_program,
        &amm_keys.market,
        &user_owner,
        &user_coin,
        &user_pc,
        &user_lp,
        amm_keys.nonce,
        open_time,
        pc_amount,
        coin_amount,
    )?;
    Ok(amm_pool_init_instruction)
}

pub fn deposit(
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbook::MarketPubkeys,
    user_owner: &Pubkey,
    user_coin: &Pubkey,
    user_pc: &Pubkey,
    user_lp: &Pubkey,
    max_coin_amount: u64,
    max_pc_amount: u64,
    base_side: u64, //0: base coin; 1: base pc
) -> Result<Instruction> {
    let deposit_instruction = raydium_amm::instruction::deposit(
        &amm_program,
        &amm_keys.amm_pool,
        &amm_keys.amm_authority,
        &amm_keys.amm_open_order,
        &amm_keys.amm_target,
        &amm_keys.amm_lp_mint,
        &amm_keys.amm_coin_vault,
        &amm_keys.amm_pc_vault,
        &amm_keys.market,
        &market_keys.event_q,
        &user_coin,
        &user_pc,
        &user_lp,
        &user_owner,
        max_coin_amount,
        max_pc_amount,
        base_side,
    )?;
    Ok(deposit_instruction)
}

pub fn withdraw(
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbook::MarketPubkeys,
    user_owner: &Pubkey,
    user_coin: &Pubkey,
    user_pc: &Pubkey,
    user_lp: &Pubkey,
    withdraw_lp_amount: u64,
) -> Result<Instruction> {
    let withdraw_instruction = raydium_amm::instruction::withdraw(
        &amm_program,
        &amm_keys.amm_pool,
        &amm_keys.amm_authority,
        &amm_keys.amm_open_order,
        &amm_keys.amm_target,
        &amm_keys.amm_lp_mint,
        &amm_keys.amm_coin_vault,
        &amm_keys.amm_pc_vault,
        &amm_keys.market_program,
        &amm_keys.market,
        &market_keys.coin_vault,
        &market_keys.pc_vault,
        &market_keys.vault_signer_key,
        user_lp,
        user_coin,
        user_pc,
        user_owner,
        &market_keys.event_q,
        &market_keys.bids,
        &market_keys.asks,
        None,
        withdraw_lp_amount,
    )?;
    Ok(withdraw_instruction)
}

pub fn swap(
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbook::MarketPubkeys,
    user_owner: &Pubkey,
    user_source: &Pubkey,
    user_destination: &Pubkey,
    amount_specified: u64,
    other_amount_threshold: u64,
    swap_base_in: bool,
) -> Result<Instruction> {
    let swap_instruction = if swap_base_in {
        raydium_amm::instruction::swap_base_in(
            &amm_program,
            &amm_keys.amm_pool,
            &amm_keys.amm_authority,
            &amm_keys.amm_open_order,
            &amm_keys.amm_coin_vault,
            &amm_keys.amm_pc_vault,
            &amm_keys.market_program,
            &amm_keys.market,
            &market_keys.bids,
            &market_keys.asks,
            &market_keys.event_q,
            &market_keys.coin_vault,
            &market_keys.pc_vault,
            &market_keys.vault_signer_key,
            user_source,
            user_destination,
            user_owner,
            amount_specified,
            other_amount_threshold,
        )?
    } else {
        raydium_amm::instruction::swap_base_out(
            &amm_program,
            &amm_keys.amm_pool,
            &amm_keys.amm_authority,
            &amm_keys.amm_open_order,
            &amm_keys.amm_coin_vault,
            &amm_keys.amm_pc_vault,
            &amm_keys.market_program,
            &amm_keys.market,
            &market_keys.bids,
            &market_keys.asks,
            &market_keys.event_q,
            &market_keys.coin_vault,
            &market_keys.pc_vault,
            &market_keys.vault_signer_key,
            user_source,
            user_destination,
            user_owner,
            other_amount_threshold,
            amount_specified,
        )?
    };

    Ok(swap_instruction)
}
