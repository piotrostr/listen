use crate::amm;
use crate::common;
use amm::{openbook, utils::AmmKeys};
use anyhow::Result;
use arrayref::array_ref;
use common::rpc;
use raydium_amm::math::{CheckedCeilDiv, U128};
use safe_transmute::{to_bytes::transmute_to_bytes, transmute_one_pedantic};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account_info::IntoAccountInfo, program_pack::Pack};
use solana_sdk::{
    commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey,
    transaction::Transaction,
};

use super::CalculateMethod;
use super::CalculateResult;

pub const TEN_THOUSAND: u64 = 10000;

fn max_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    input_amount
        .checked_mul(slippage_bps.checked_add(TEN_THOUSAND).unwrap())
        .unwrap()
        .checked_div(TEN_THOUSAND)
        .unwrap()
}

fn min_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    input_amount
        .checked_mul(TEN_THOUSAND.checked_sub(slippage_bps).unwrap())
        .unwrap()
        .checked_div(TEN_THOUSAND)
        .unwrap()
}

// pool_vault_amount = vault_amount + open_orders.native_total + partial filled without consumed - amm.need_take
pub async fn calculate_pool_vault_amounts(
    client: &RpcClient,
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbook::MarketPubkeys,
    calculate_method: CalculateMethod,
) -> Result<CalculateResult> {
    let result = match calculate_method {
        CalculateMethod::CalculateWithLoadAccount => {
            // reload accounts data to calculate amm pool vault amount
            // get multiple accounts at the same time to ensure data consistency
            let load_pubkeys = vec![
                *amm_pool,
                amm_keys.amm_target,
                amm_keys.amm_pc_vault,
                amm_keys.amm_coin_vault,
                amm_keys.amm_open_order,
                amm_keys.market,
                *market_keys.event_q,
            ];
            let rsps =
                rpc::get_multiple_accounts(client, &load_pubkeys).await?;
            let accounts = array_ref![rsps, 0, 7];
            let [amm_account, amm_target_account, amm_pc_vault_account, amm_coin_vault_account, amm_open_orders_account, market_account, market_event_q_account] =
                accounts;
            let amm: raydium_amm::state::AmmInfo = transmute_one_pedantic::<
                raydium_amm::state::AmmInfo,
            >(
                transmute_to_bytes(&amm_account.as_ref().unwrap().clone().data),
            )
            .map_err(|e| e.without_src())?;
            let _amm_target: raydium_amm::state::TargetOrders =
                transmute_one_pedantic::<raydium_amm::state::TargetOrders>(
                    transmute_to_bytes(
                        &amm_target_account.as_ref().unwrap().clone().data,
                    ),
                )
                .map_err(|e| e.without_src())?;
            let amm_pc_vault = spl_token::state::Account::unpack(
                &amm_pc_vault_account.as_ref().unwrap().clone().data,
            )
            .unwrap();
            let amm_coin_vault = spl_token::state::Account::unpack(
                &amm_coin_vault_account.as_ref().unwrap().clone().data,
            )
            .unwrap();
            let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                if raydium_amm::state::AmmStatus::from_u64(amm.status)
                    .orderbook_permission()
                {
                    let amm_open_orders_account =
                        &mut amm_open_orders_account.as_ref().unwrap().clone();
                    let market_account =
                        &mut market_account.as_ref().unwrap().clone();
                    let market_event_q_account =
                        &mut market_event_q_account.as_ref().unwrap().clone();

                    let amm_open_orders_info =
                        (&amm.open_orders, amm_open_orders_account)
                            .into_account_info();
                    let market_account_info =
                        (&amm.market, market_account).into_account_info();
                    let market_event_queue_info =
                        (&(*market_keys.event_q), market_event_q_account)
                            .into_account_info();

                    let amm_authority = Pubkey::find_program_address(
                        &[raydium_amm::processor::AUTHORITY_AMM],
                        &amm_program,
                    )
                    .0;
                    let lamports = &mut 0;
                    let data = &mut [0u8];
                    let owner = Pubkey::default();
                    let amm_authority_info =
                        solana_program::account_info::AccountInfo::new(
                            &amm_authority,
                            false,
                            false,
                            lamports,
                            data,
                            &owner,
                            false,
                            0,
                        );
                    let (market_state, open_orders) =
                        raydium_amm::processor::Processor::load_serum_market_order(
                            &market_account_info,
                            &amm_open_orders_info,
                            &amm_authority_info,
                            &amm,
                            false,
                        )?;
                    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                        raydium_amm::math::Calculator::calc_total_without_take_pnl(
                            amm_pc_vault.amount,
                            amm_coin_vault.amount,
                            &open_orders,
                            &amm,
                            &market_state,
                            &market_event_queue_info,
                            &amm_open_orders_info,
                        )?;
                    (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
                } else {
                    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                        raydium_amm::math::Calculator::calc_total_without_take_pnl_no_orderbook(
                            amm_pc_vault.amount,
                            amm_coin_vault.amount,
                            &amm,
                        )?;
                    (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
                };

            // // deduct pnl
            // let (pool_pc_vault_without_pnl, pool_coin_vault_without_pnl) = pool_vault_deduct_pnl(
            //     amm_pool_pc_vault_amount,
            //     amm_pool_coin_vault_amount,
            //     &mut amm,
            //     &amm_target,
            // )?;
            CalculateResult {
                pool_pc_vault_amount: amm_pool_pc_vault_amount,
                pool_coin_vault_amount: amm_pool_coin_vault_amount,
                pool_lp_amount: amm.lp_amount,
                swap_fee_numerator: amm.fees.swap_fee_numerator,
                swap_fee_denominator: amm.fees.swap_fee_denominator,
            }
        }
        CalculateMethod::Simulate(fee_payer) => {
            let amm = rpc::get_account::<raydium_amm::state::AmmInfo>(
                client, amm_pool,
            )
            .await?
            .unwrap();
            let simulate_pool_info_instruction =
                raydium_amm::instruction::simulate_get_pool_info(
                    amm_program,
                    amm_pool,
                    &amm_keys.amm_authority,
                    &amm_keys.amm_open_order,
                    &amm_keys.amm_coin_vault,
                    &amm_keys.amm_pc_vault,
                    &amm_keys.amm_lp_mint,
                    &amm_keys.market,
                    &market_keys.event_q,
                    None,
                )?;
            let mut message = Message::new(
                &[simulate_pool_info_instruction],
                Some(&fee_payer),
            );
            message.recent_blockhash = client.get_latest_blockhash().await?;
            let txn = Transaction::new_unsigned(message);
            let result = rpc::simulate_transaction(
                &client,
                &txn,
                false,
                CommitmentConfig::confirmed(),
            )
            .await?;
            // println!("{:#?}", result);
            let mut ret = raydium_amm::state::GetPoolData::default();
            if result.value.err.is_none() {
                if let Some(logs) = result.value.logs {
                    for log in logs {
                        if let Some(_) = log.find("GetPoolData: ") {
                            let begin = log.find("{").unwrap();
                            let end = log.rfind("}").unwrap() + 1;
                            let json_str = log.get(begin..end).unwrap();
                            ret = raydium_amm::state::GetPoolData::from_json(
                                json_str,
                            )
                        }
                    }
                }
            }
            CalculateResult {
                pool_pc_vault_amount: ret.pool_pc_amount,
                pool_coin_vault_amount: ret.pool_coin_amount,
                pool_lp_amount: ret.pool_lp_supply,
                swap_fee_numerator: amm.fees.swap_fee_numerator,
                swap_fee_denominator: amm.fees.swap_fee_denominator,
            }
        }
    };
    Ok(result)
}

pub fn pool_vault_deduct_pnl(
    pc_vault_amount_with_pnl: u64,
    coin_vault_amount_with_pnl: u64,
    amm: &mut raydium_amm::state::AmmInfo,
    target: &raydium_amm::state::TargetOrders,
) -> Result<(u64, u64)> {
    let mut pc_vault_amount_with_pnl = pc_vault_amount_with_pnl;
    let mut coin_vault_amount_with_pnl = coin_vault_amount_with_pnl;
    let x = raydium_amm::math::Calculator::normalize_decimal_v2(
        pc_vault_amount_with_pnl,
        amm.pc_decimals,
        amm.sys_decimal_value,
    );
    let y = raydium_amm::math::Calculator::normalize_decimal_v2(
        coin_vault_amount_with_pnl,
        amm.coin_decimals,
        amm.sys_decimal_value,
    );
    // calc and update pnl with adjust vault amount
    let (_delta_x, _delta_y) =
        raydium_amm::processor::Processor::calc_take_pnl(
            target,
            amm,
            &mut pc_vault_amount_with_pnl,
            &mut coin_vault_amount_with_pnl,
            x.as_u128().into(),
            y.as_u128().into(),
        )
        .unwrap();

    Ok((pc_vault_amount_with_pnl, coin_vault_amount_with_pnl))
}

fn deposit_exact_amount(
    pc_vault_amount_without_pnl: u64,
    coin_vault_amount_without_pnl: u64,
    input_amount: u64,
    base_side: u64,
) -> Result<u64> {
    // calc deposit amount
    let invariant = raydium_amm::math::InvariantToken {
        token_coin: coin_vault_amount_without_pnl,
        token_pc: pc_vault_amount_without_pnl,
    };
    match base_side {
        0 => {
            // input amount is coin
            let another_amount = invariant
                .exchange_coin_to_pc(
                    input_amount,
                    raydium_amm::math::RoundDirection::Ceiling,
                )
                .unwrap();
            Ok(another_amount)
        }
        _ => {
            // input amount is pc
            let another_amount = invariant
                .exchange_pc_to_coin(
                    input_amount,
                    raydium_amm::math::RoundDirection::Ceiling,
                )
                .unwrap();
            Ok(another_amount)
        }
    }
}

fn withdraw_exact_amounts(
    pc_vault_amount_without_pnl: u64,
    coin_vault_amount_without_pnl: u64,
    pool_lp_amount: u64,
    withdraw_lp_amount: u64,
) -> Result<(u64, u64)> {
    // calc withdraw amount
    let invariant = raydium_amm::math::InvariantPool {
        token_input: withdraw_lp_amount,
        token_total: pool_lp_amount,
    };
    let pc_amount = invariant
        .exchange_pool_to_token(
            pc_vault_amount_without_pnl,
            raydium_amm::math::RoundDirection::Floor,
        )
        .unwrap();
    let coin_amount = invariant
        .exchange_pool_to_token(
            coin_vault_amount_without_pnl,
            raydium_amm::math::RoundDirection::Floor,
        )
        .unwrap();

    Ok((pc_amount, coin_amount))
}

fn swap_exact_amount(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: raydium_amm::math::SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
) -> Result<u64> {
    let other_amount_threshold = if swap_base_in {
        let swap_fee = U128::from(amount_specified)
            .checked_mul(swap_fee_numerator.into())
            .unwrap()
            .checked_ceil_div(swap_fee_denominator.into())
            .unwrap()
            .0;
        let swap_in_after_deduct_fee =
            U128::from(amount_specified).checked_sub(swap_fee).unwrap();
        let swap_amount_out =
            raydium_amm::math::Calculator::swap_token_amount_base_in(
                swap_in_after_deduct_fee,
                pc_vault_amount.into(),
                coin_vault_amount.into(),
                swap_direction,
            )
            .as_u64();
        swap_amount_out
    } else {
        let swap_in_before_add_fee =
            raydium_amm::math::Calculator::swap_token_amount_base_out(
                amount_specified.into(),
                pc_vault_amount.into(),
                coin_vault_amount.into(),
                swap_direction,
            );
        let swap_in_after_add_fee = swap_in_before_add_fee
            .checked_mul(swap_fee_denominator.into())
            .unwrap()
            .checked_ceil_div(
                (swap_fee_denominator
                    .checked_sub(swap_fee_numerator)
                    .unwrap())
                .into(),
            )
            .unwrap()
            .0
            .as_u64();

        swap_in_after_add_fee
    };

    Ok(other_amount_threshold)
}

pub fn deposit_amount_with_slippage(
    pc_vault_amount_without_pnl: u64,
    coin_vault_amount_without_pnl: u64,
    input_amount: u64,
    base_side: u64,
    slippage_bps: u64,
) -> Result<(u64, u64)> {
    let another_amount = deposit_exact_amount(
        pc_vault_amount_without_pnl,
        coin_vault_amount_without_pnl,
        input_amount,
        base_side,
    )?;
    match base_side {
        0 => {
            let max_coin_amout = input_amount;
            let max_pc_amount =
                max_amount_with_slippage(another_amount, slippage_bps);
            return Ok((max_coin_amout, max_pc_amount));
        }
        _ => {
            let max_coin_amount =
                max_amount_with_slippage(another_amount, slippage_bps);
            let max_pc_amount = input_amount;
            return Ok((max_coin_amount, max_pc_amount));
        }
    }
}

pub fn withdraw_amounts_with_slippage(
    pc_vault_amount_without_pnl: u64,
    coin_vault_amount_without_pnl: u64,
    pool_lp_amount: u64,
    withdraw_lp_amount: u64,
    _slippage_bps: u64, // not used yet
) -> Result<(u64, u64)> {
    let (pc_amount, coin_amount) = withdraw_exact_amounts(
        pc_vault_amount_without_pnl,
        coin_vault_amount_without_pnl,
        pool_lp_amount,
        withdraw_lp_amount,
    )?;

    Ok((pc_amount, coin_amount))
}

pub fn swap_with_slippage(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: amm::utils::SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
    slippage_bps: u64,
) -> Result<u64> {
    let other_amount_threshold = swap_exact_amount(
        pc_vault_amount,
        coin_vault_amount,
        swap_fee_numerator,
        swap_fee_denominator,
        match swap_direction {
            amm::utils::SwapDirection::Coin2PC => {
                raydium_amm::math::SwapDirection::Coin2PC
            }
            amm::utils::SwapDirection::PC2Coin => {
                raydium_amm::math::SwapDirection::PC2Coin
            }
        },
        amount_specified,
        swap_base_in,
    )?;
    let other_amount_threshold = if swap_base_in {
        // min out
        min_amount_with_slippage(other_amount_threshold, slippage_bps)
    } else {
        // max in
        max_amount_with_slippage(other_amount_threshold, slippage_bps)
    };
    Ok(other_amount_threshold)
}