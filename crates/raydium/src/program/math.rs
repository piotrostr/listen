//! Defines PreciseNumber, a U256 wrapper with float-like operations

use crate::{program::error::AmmError, program::state::AmmInfo};
use num_traits::CheckedDiv;
use crate::serum_dex::{
    matching::Side,
    state::{EventView, MarketState, OpenOrders, ToAlignedBytes},
};
use solana_sdk::{account_info::AccountInfo, msg};
use std::{cmp::Eq, convert::identity, convert::TryInto};
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}
construct_uint! {
    pub struct U128(2);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin = 1u64,
    /// Input token coin, output token pc
    Coin2PC = 2u64,
}

/// The direction to round.  Used for pool token to trading token conversions to
/// avoid losing value on any deposit or withdrawal.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundDirection {
    /// Floor the value, ie. 1.9 => 1, 1.1 => 1, 1.5 => 1
    Floor,
    /// Ceiling the value, ie. 1.9 => 2, 1.1 => 2, 1.5 => 2
    Ceiling,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Calculator {}

impl Calculator {
    pub fn to_u128(val: u64) -> Result<u128, AmmError> {
        val.try_into().map_err(|_| AmmError::ConversionFailure)
    }

    pub fn to_u64(val: u128) -> Result<u64, AmmError> {
        val.try_into().map_err(|_| AmmError::ConversionFailure)
    }

    pub fn calc_x_power(
        last_x: U256,
        last_y: U256,
        current_x: U256,
        current_y: U256,
    ) -> U256 {
        // must be use u256, because u128 may be overflow
        let x_power = last_x
            .checked_mul(last_y)
            .unwrap()
            .checked_mul(current_x)
            .unwrap()
            .checked_div(current_y)
            .unwrap();
        x_power
    }

    // out: 0, 1, 2, 3, 5, 8, 13, 21, 34, 55
    pub fn fibonacci(order_num: u64) -> Vec<u64> {
        let mut fb = Vec::new();
        for i in 0..order_num {
            if i == 0 {
                fb.push(0u64);
            } else if i == 1 {
                fb.push(1u64);
            } else if i == 2 {
                fb.push(2u64);
            } else {
                let ret = fb[(i - 1u64) as usize] + fb[(i - 2u64) as usize];
                fb.push(ret);
            };
        }
        return fb;
    }

    pub fn normalize_decimal(
        val: u64,
        native_decimal: u64,
        sys_decimal_value: u64,
    ) -> u64 {
        // e.g., amm.sys_decimal_value is 10**6, native_decimal is 10**9, price is 1.23, this function will convert (1.23*10**9) -> (1.23*10**6)
        //let ret:u64 = val.checked_mul(amm.sys_decimal_value).unwrap().checked_div((10 as u64).pow(native_decimal.into())).unwrap();
        let ret_mut = (U128::from(val))
            .checked_mul(sys_decimal_value.into())
            .unwrap();
        let ret = Self::to_u64(
            ret_mut
                .checked_div(
                    U128::from(10).checked_pow(native_decimal.into()).unwrap(),
                )
                .unwrap()
                .as_u128(),
        )
        .unwrap();
        ret
    }

    pub fn restore_decimal(
        val: U128,
        native_decimal: u64,
        sys_decimal_value: u64,
    ) -> U128 {
        // e.g., amm.sys_decimal_value is 10**6, native_decimal is 10**9, price is 1.23, this function will convert (1.23*10**6) -> (1.23*10**9)
        // let ret:u64 = val.checked_mul((10 as u64).pow(native_decimal.into())).unwrap().checked_div(amm.sys_decimal_value).unwrap();
        let ret_mut = val
            .checked_mul(
                U128::from(10).checked_pow(native_decimal.into()).unwrap(),
            )
            .unwrap();
        let ret = ret_mut.checked_div(sys_decimal_value.into()).unwrap();
        ret
    }

    pub fn normalize_decimal_v2(
        val: u64,
        native_decimal: u64,
        sys_decimal_value: u64,
    ) -> U128 {
        // e.g., amm.sys_decimal_value is 10**6, native_decimal is 10**9, price is 1.23, this function will convert (1.23*10**9) -> (1.23*10**6)
        //let ret:u64 = val.checked_mul(amm.sys_decimal_value).unwrap().checked_div((10 as u64).pow(native_decimal.into())).unwrap();
        let ret_mut = (U128::from(val))
            .checked_mul(sys_decimal_value.into())
            .unwrap();
        let ret = ret_mut
            .checked_div(
                U128::from(10).checked_pow(native_decimal.into()).unwrap(),
            )
            .unwrap();
        ret
    }

    pub fn floor_lot(val: u64, lot_size: u64) -> u64 {
        // all numbers are in normalized decimal already
        let unit: u64 = val.checked_div(lot_size).unwrap();
        let ret: u64 = unit.checked_mul(lot_size).unwrap();
        ret
    }

    pub fn ceil_lot(val: u64, lot_size: u64) -> u64 {
        let unit: u128 =
            (val as u128).checked_ceil_div(lot_size as u128).unwrap().0;
        let ret: u64 =
            Self::to_u64(unit).unwrap().checked_mul(lot_size).unwrap();
        ret
    }

    /*
        o_pls = pls * (cls * pc_dec) / (dec * c_dec) => convert_out_pc_lot_sz
        pls = dec * o_pls * c_dec / (cls * pc_dec)  => convert_in_pc_lot_sz

        c_sz = o_c_sz * cls * dec / c_dec => convert_in_vol
        o_c_sz = c_sz * c_dec / (cls * dec) => convert_out_vol

        p = o_p * pls => convert_in_price
        o_p = p / pls => convert_out_price
    */

    // convert internal pc_lot_size -> srm pc_lot_size
    pub fn convert_out_pc_lot_size(
        pc_decimals: u8,
        coin_decimals: u8,
        pc_lot_size: u64,
        coin_lot_size: u64,
        sys_decimal_value: u64,
    ) -> u64 {
        let native_lot_size = Self::to_u64(
            ((U128::from(pc_lot_size)
                * U128::from(coin_lot_size)
                * (U128::from(10).checked_pow(pc_decimals.into()).unwrap()))
                / (U128::from(sys_decimal_value)
                    * (U128::from(10)
                        .checked_pow(coin_decimals.into())
                        .unwrap())))
            .as_u128(),
        )
        .unwrap();
        native_lot_size
    }

    // convert srm pc_lot_size -> internal pc_lot_size
    pub fn convert_in_pc_lot_size(
        pc_decimals: u8,
        coin_decimals: u8,
        pc_lot_size: u64,
        coin_lot_size: u64,
        sys_decimal_value: u64,
    ) -> u64 {
        let native_lot_size = Self::to_u64(
            (U128::from(pc_lot_size)
                .checked_mul(sys_decimal_value.into())
                .unwrap()
                .checked_mul(
                    U128::from(10).checked_pow(coin_decimals.into()).unwrap(),
                )
                .unwrap())
            .checked_div(
                U128::from(coin_lot_size)
                    .checked_mul(
                        U128::from(10)
                            .checked_pow(pc_decimals.into())
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
            .as_u128(),
        )
        .unwrap();
        native_lot_size
    }

    // convert srm price -> internal price
    pub fn convert_in_price(val: u64, pc_lot_size: u64) -> u64 {
        let price = val.checked_mul(pc_lot_size).unwrap();
        price
    }

    // convert internal price -> srm price
    pub fn convert_price_out(val: u64, pc_lot_size: u64) -> u64 {
        let price = val.checked_div(pc_lot_size).unwrap();
        price
    }

    // convert srm coin size -> internal coin size
    pub fn convert_in_vol(
        val: u64,
        coin_decimal: u64,
        coin_lot_size: u64,
        sys_decimal_value: u64,
    ) -> u64 {
        let volume: U128 = U128::from(val)
            .checked_mul(coin_lot_size.into())
            .unwrap()
            .checked_mul(sys_decimal_value.into())
            .unwrap()
            .checked_div(
                U128::from(10).checked_pow(coin_decimal.into()).unwrap(),
            )
            .unwrap();
        let ret: u64 = Self::to_u64(volume.as_u128()).unwrap();
        ret
    }

    // convert internal coin size -> srm coin size
    pub fn convert_vol_out(
        val: u64,
        coin_decimal: u64,
        coin_lot_size: u64,
        sys_decimal_value: u64,
    ) -> u64 {
        let volume: U128 = U128::from(val)
            .checked_mul(
                U128::from(10).checked_pow(coin_decimal.into()).unwrap(),
            )
            .unwrap()
            .checked_div(
                U128::from(coin_lot_size)
                    .checked_mul(sys_decimal_value.into())
                    .unwrap(),
            )
            .unwrap();
        let ret: u64 = Self::to_u64(volume.as_u128()).unwrap();
        ret
    }

    pub fn calc_exact_vault_in_serum<'a>(
        open_orders: &'a OpenOrders,
        market_state: &'a Box<MarketState>,
        event_q_account: &'a AccountInfo,
        amm_open_account: &'a AccountInfo,
    ) -> Result<(u64, u64), AmmError> {
        let event_q =
            market_state.load_event_queue_mut(event_q_account).unwrap();
        let mut native_pc_total = open_orders.native_pc_total;
        let mut native_coin_total = open_orders.native_coin_total;
        msg!("calc_exact len:{}", event_q.len());
        for event in event_q.iter() {
            if identity(event.owner)
                != (*amm_open_account.key).to_bytes()
            {
                continue;
            }
            // msg!("{:?}", event.as_view().unwrap());
            match event.as_view().unwrap() {
                EventView::Fill {
                    side,
                    maker,
                    native_qty_paid,
                    native_qty_received,
                    native_fee_or_rebate: _,
                    fee_tier: _,
                    order_id: _,
                    owner: _,
                    owner_slot: _,
                    client_order_id: _,
                } => {
                    match side {
                        Side::Bid if maker => {
                            native_pc_total -= native_qty_paid;
                            native_coin_total += native_qty_received;
                        }
                        Side::Ask if maker => {
                            native_coin_total -= native_qty_paid;
                            native_pc_total += native_qty_received;
                        }
                        _ => (),
                    };
                }
                _ => {
                    continue;
                }
            }
        }
        Ok((native_pc_total, native_coin_total))
    }

    pub fn calc_total_without_take_pnl<'a>(
        pc_amount: u64,
        coin_amount: u64,
        open_orders: &'a OpenOrders,
        amm: &'a AmmInfo,
        market_state: &'a Box<MarketState>,
        event_q_account: &'a AccountInfo,
        amm_open_account: &'a AccountInfo,
    ) -> Result<(u64, u64), AmmError> {
        let (pc_total_in_serum, coin_total_in_serum) =
            Self::calc_exact_vault_in_serum(
                open_orders,
                market_state,
                event_q_account,
                amm_open_account,
            )?;

        let total_pc_without_take_pnl = pc_amount
            .checked_add(pc_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_add(coin_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

    pub fn calc_total_without_take_pnl_no_orderbook<'a>(
        pc_amount: u64,
        coin_amount: u64,
        amm: &'a AmmInfo,
    ) -> Result<(u64, u64), AmmError> {
        let total_pc_without_take_pnl = pc_amount
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

    pub fn get_max_buy_size_at_price(
        price: u64,
        x: u128,
        y: u128,
        amm: &AmmInfo,
    ) -> u64 {
        // max_size = x / (1.0025 * price) - y
        let price_with_fee = U128::from(price)
            .checked_mul(U128::from(
                amm.fees.trade_fee_denominator + amm.fees.trade_fee_numerator,
            ))
            .unwrap()
            .checked_div(U128::from(amm.fees.trade_fee_denominator))
            .unwrap();
        let mut max_size = U128::from(x)
            .checked_mul(amm.sys_decimal_value.into())
            .unwrap()
            .checked_div(price_with_fee)
            .unwrap();
        max_size = max_size.saturating_sub(y.into());
        Self::to_u64(max_size.as_u128()).unwrap()
    }

    pub fn get_max_sell_size_at_price(
        price: u64,
        x: u128,
        y: u128,
        amm: &AmmInfo,
    ) -> u64 {
        // let max_size = y - x / (p / 1.0025)
        let price_with_fee = U128::from(price)
            .checked_mul(amm.fees.trade_fee_denominator.into())
            .unwrap()
            .checked_div(U128::from(
                amm.fees.trade_fee_denominator + amm.fees.trade_fee_numerator,
            ))
            .unwrap();
        let second_part = U128::from(x)
            .checked_mul(amm.sys_decimal_value.into())
            .unwrap()
            .checked_div(price_with_fee.into())
            .unwrap();

        let max_size = U128::from(y).saturating_sub(second_part);
        Self::to_u64(max_size.as_u128()).unwrap()
    }

    pub fn swap_token_amount_base_in(
        amount_in: U128,
        total_pc_without_take_pnl: U128,
        total_coin_without_take_pnl: U128,
        swap_direction: SwapDirection,
    ) -> U128 {
        let amount_out;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_out = pc - coin * pc / (coin + amount_in)
                // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
                // => amount_out =  pc * amount_in / (coin + amount_in)
                let denominator = total_coin_without_take_pnl
                    .checked_add(amount_in)
                    .unwrap();
                amount_out = total_pc_without_take_pnl
                    .checked_mul(amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)
                let denominator =
                    total_pc_without_take_pnl.checked_add(amount_in).unwrap();
                amount_out = total_coin_without_take_pnl
                    .checked_mul(amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
            }
        }
        return amount_out;
    }

    pub fn swap_token_amount_base_out(
        amount_out: U128,
        total_pc_without_take_pnl: U128,
        total_coin_without_take_pnl: U128,
        swap_direction: SwapDirection,
    ) -> U128 {
        let amount_in;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_in = coin * pc / (pc - amount_out) - coin
                // => amount_in = (coin * pc - pc * coin + amount_out * coin) / (pc - amount_out)
                // => amount_in = (amount_out * coin) / (pc - amount_out)
                let denominator =
                    total_pc_without_take_pnl.checked_sub(amount_out).unwrap();
                amount_in = total_coin_without_take_pnl
                    .checked_mul(amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)

                // => amount_in = coin * pc / (coin - amount_out) - pc
                // => amount_in = (coin * pc - pc * coin + pc * amount_out) / (coin - amount_out)
                // => amount_in = (pc * amount_out) / (coin - amount_out)
                let denominator = total_coin_without_take_pnl
                    .checked_sub(amount_out)
                    .unwrap();
                amount_in = total_pc_without_take_pnl
                    .checked_mul(amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
            }
        }
        return amount_in;
    }
}

/// The invariant calculator.
pub struct InvariantToken {
    /// Token coin
    pub token_coin: u64,
    /// Token pc
    pub token_pc: u64,
}

impl InvariantToken {
    /// Exchange rate
    pub fn exchange_coin_to_pc(
        &self,
        token_coin: u64,
        round_direction: RoundDirection,
    ) -> Option<u64> {
        Some(if round_direction == RoundDirection::Floor {
            U128::from(token_coin)
                .checked_mul(self.token_pc.into())
                .unwrap()
                .checked_div(self.token_coin.into())
                .unwrap()
                .as_u64()
        } else {
            U128::from(token_coin)
                .checked_mul(self.token_pc.into())
                .unwrap()
                .checked_ceil_div(self.token_coin.into())
                .unwrap()
                .0
                .as_u64()
        })
    }

    /// Exchange rate
    pub fn exchange_pc_to_coin(
        &self,
        token_pc: u64,
        round_direction: RoundDirection,
    ) -> Option<u64> {
        Some(if round_direction == RoundDirection::Floor {
            U128::from(token_pc)
                .checked_mul(self.token_coin.into())
                .unwrap()
                .checked_div(self.token_pc.into())
                .unwrap()
                .as_u64()
        } else {
            U128::from(token_pc)
                .checked_mul(self.token_coin.into())
                .unwrap()
                .checked_ceil_div(self.token_pc.into())
                .unwrap()
                .0
                .as_u64()
        })
    }
}

/// The invariant calculator.
pub struct InvariantPool {
    /// Token input
    pub token_input: u64,
    /// Token total
    pub token_total: u64,
}
impl InvariantPool {
    /// Exchange rate
    pub fn exchange_pool_to_token(
        &self,
        token_total_amount: u64,
        round_direction: RoundDirection,
    ) -> Option<u64> {
        Some(if round_direction == RoundDirection::Floor {
            U128::from(token_total_amount)
                .checked_mul(self.token_input.into())
                .unwrap()
                .checked_div(self.token_total.into())
                .unwrap()
                .as_u64()
        } else {
            U128::from(token_total_amount)
                .checked_mul(self.token_input.into())
                .unwrap()
                .checked_ceil_div(self.token_total.into())
                .unwrap()
                .0
                .as_u64()
        })
    }
    /// Exchange rate
    pub fn exchange_token_to_pool(
        &self,
        pool_total_amount: u64,
        round_direction: RoundDirection,
    ) -> Option<u64> {
        Some(if round_direction == RoundDirection::Floor {
            U128::from(pool_total_amount)
                .checked_mul(self.token_input.into())
                .unwrap()
                .checked_div(self.token_total.into())
                .unwrap()
                .as_u64()
        } else {
            U128::from(pool_total_amount)
                .checked_mul(self.token_input.into())
                .unwrap()
                .checked_ceil_div(self.token_total.into())
                .unwrap()
                .0
                .as_u64()
        })
    }
}

/// Perform a division that does not truncate value from either side, returning
/// the (quotient, divisor) as a tuple
///
/// When dividing integers, we are often left with a remainder, which can
/// cause information to be lost.  By checking for a remainder, adjusting
/// the quotient, and recalculating the divisor, this provides the most fair
/// calculation.
///
/// For example, 400 / 32 = 12, with a remainder cutting off 0.5 of amount.
/// If we simply ceiling the quotient to 13, then we're saying 400 / 32 = 13, which
/// also cuts off value.  To improve this result, we calculate the other way
/// around and again check for a remainder: 400 / 13 = 30, with a remainder of
/// 0.77, and we ceiling that value again.  This gives us a final calculation
/// of 400 / 31 = 13, which provides a ceiling calculation without cutting off
/// more value than needed.
///
/// This calculation fails if the divisor is larger than the dividend, to avoid
/// having a result like: 1 / 1000 = 1.
pub trait CheckedCeilDiv: Sized {
    /// Perform ceiling division
    fn checked_ceil_div(&self, rhs: Self) -> Option<(Self, Self)>;
}

impl CheckedCeilDiv for u128 {
    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(&rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        if quotient == 0 {
            // return None;
            if self.checked_mul(2 as u128)? >= rhs {
                return Some((1, 0));
            } else {
                return Some((0, 0));
            }
        }

        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > 0 {
            quotient = quotient.checked_add(1)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(&quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > 0 {
                rhs = rhs.checked_add(1)?;
            }
        }
        Some((quotient, rhs))
    }
}

impl CheckedCeilDiv for U128 {
    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        let zero = U128::from(0);
        let one = U128::from(1);
        if quotient.is_zero() {
            // return None;
            if self.checked_mul(U128::from(2))? >= rhs {
                return Some((one, zero));
            } else {
                return Some((zero, zero));
            }
        }

        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > zero {
            quotient = quotient.checked_add(one)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > zero {
                rhs = rhs.checked_add(one)?;
            }
        }
        Some((quotient, rhs))
    }
}
