use crate::common;
use anyhow::Result;

use common::rpc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Copy, Debug)]
pub struct AmmKeys {
    pub amm_pool: Pubkey,
    pub amm_coin_mint: Pubkey,
    pub amm_pc_mint: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_target: Pubkey,
    pub amm_coin_vault: Pubkey,
    pub amm_pc_vault: Pubkey,
    pub amm_lp_mint: Pubkey,
    pub amm_open_order: Pubkey,
    pub market_program: Pubkey,
    pub market: Pubkey,
    pub nonce: u8,
}

pub enum CalculateMethod {
    CalculateWithLoadAccount,
    Simulate(Pubkey),
}

pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin,
    /// Input token coin, output token pc
    Coin2PC,
}

#[derive(Clone, Copy, Debug)]
pub struct CalculateResult {
    pub pool_pc_vault_amount: u64,
    pub pool_coin_vault_amount: u64,
    pub pool_lp_amount: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

// only use for initialize_amm_pool, because the keys of some amm pools are not used in this way.
pub fn get_amm_pda_keys(
    amm_program: &Pubkey,
    market_program: &Pubkey,
    market: &Pubkey,
    coin_mint: &Pubkey,
    pc_mint: &Pubkey,
) -> Result<AmmKeys> {
    let amm_pool =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::AMM_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;
    let (amm_authority, nonce) = Pubkey::find_program_address(
        &[raydium_amm::processor::AUTHORITY_AMM],
        &amm_program,
    );
    let amm_open_order =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::OPEN_ORDER_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;
    let amm_lp_mint =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::LP_MINT_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;
    let amm_coin_vault =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::COIN_VAULT_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;
    let amm_pc_vault =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::PC_VAULT_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;
    let amm_target =
        raydium_amm::processor::get_associated_address_and_bump_seed(
            &amm_program,
            &market,
            raydium_amm::processor::TARGET_ASSOCIATED_SEED,
            &amm_program,
        )
        .0;

    Ok(AmmKeys {
        amm_pool,
        amm_target,
        amm_coin_vault,
        amm_pc_vault,
        amm_lp_mint,
        amm_open_order,
        amm_coin_mint: *coin_mint,
        amm_pc_mint: *pc_mint,
        amm_authority,
        market: *market,
        market_program: *market_program,
        nonce,
    })
}

pub async fn load_amm_keys(
    client: &RpcClient,
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
) -> Result<AmmKeys> {
    let amm =
        rpc::get_account::<raydium_amm::state::AmmInfo>(client, &amm_pool)
            .await?
            .unwrap();
    Ok(AmmKeys {
        amm_pool: *amm_pool,
        amm_target: amm.target_orders,
        amm_coin_vault: amm.coin_vault,
        amm_pc_vault: amm.pc_vault,
        amm_lp_mint: amm.lp_mint,
        amm_open_order: amm.open_orders,
        amm_coin_mint: amm.coin_vault_mint,
        amm_pc_mint: amm.pc_vault_mint,
        amm_authority: raydium_amm::processor::Processor::authority_id(
            amm_program,
            raydium_amm::processor::AUTHORITY_AMM,
            amm.nonce as u8,
        )?,
        market: amm.market,
        market_program: amm.market_program,
        nonce: amm.nonce as u8,
    })
}
