//! State transition types

use crate::serum_dex::state::ToAlignedBytes;
use crate::{program::error::AmmError, program::math::Calculator};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use safe_transmute::TriviallyTransmutable;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::{
    cell::{Ref, RefMut},
    convert::identity,
    convert::TryInto,
    mem::size_of,
};

pub const TEN_THOUSAND: u64 = 10000;
pub const MAX_ORDER_LIMIT: usize = 10;

pub trait Loadable: Pod {
    fn load_mut<'a>(
        account: &'a AccountInfo,
    ) -> Result<RefMut<'a, Self>, ProgramError> {
        // TODO verify if this checks for size
        Ok(RefMut::map(account.try_borrow_mut_data()?, |data| {
            from_bytes_mut(data)
        }))
    }
    fn load<'a>(
        account: &'a AccountInfo,
    ) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(account.try_borrow_data()?, |data| {
            from_bytes(data)
        }))
    }

    fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        Ok(from_bytes(data))
    }
}

macro_rules! impl_loadable {
    ($type_name:ident) => {
        unsafe impl Zeroable for $type_name {}
        unsafe impl Pod for $type_name {}
        unsafe impl TriviallyTransmutable for $type_name {}
        impl Loadable for $type_name {}
    };
}

#[cfg_attr(feature = "client", derive(Debug))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TargetOrder {
    pub price: u64,
    pub vol: u64,
}
#[cfg(target_endian = "little")]
unsafe impl Zeroable for TargetOrder {}
#[cfg(target_endian = "little")]
unsafe impl Pod for TargetOrder {}

#[cfg_attr(feature = "client", derive(Debug))]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TargetOrders {
    pub owner: [u64; 4],
    pub buy_orders: [TargetOrder; 50],
    pub padding1: [u64; 8],
    pub target_x: u128,
    pub target_y: u128,
    pub plan_x_buy: u128,
    pub plan_y_buy: u128,
    pub plan_x_sell: u128,
    pub plan_y_sell: u128,
    pub placed_x: u128,
    pub placed_y: u128,
    pub calc_pnl_x: u128,
    pub calc_pnl_y: u128,
    pub sell_orders: [TargetOrder; 50],
    pub padding2: [u64; 6],
    pub replace_buy_client_id: [u64; MAX_ORDER_LIMIT],
    pub replace_sell_client_id: [u64; MAX_ORDER_LIMIT],
    pub last_order_numerator: u64,
    pub last_order_denominator: u64,

    pub plan_orders_cur: u64,
    pub place_orders_cur: u64,

    pub valid_buy_order_num: u64,
    pub valid_sell_order_num: u64,

    pub padding3: [u64; 10],

    pub free_slot_bits: u128,
}

#[cfg(test)]
impl Default for TargetOrders {
    #[inline]
    fn default() -> TargetOrders {
        TargetOrders {
            owner: [0; 4],
            buy_orders: [TargetOrder::default(); 50],
            padding1: [0; 8],
            target_x: 0,
            target_y: 0,
            plan_x_buy: 0,
            plan_y_buy: 0,
            plan_x_sell: 0,
            plan_y_sell: 0,
            placed_x: 0,
            placed_y: 0,
            calc_pnl_x: 0,
            calc_pnl_y: 0,
            sell_orders: [TargetOrder::default(); 50],
            padding2: [0; 6],
            replace_buy_client_id: [0; MAX_ORDER_LIMIT],
            replace_sell_client_id: [0; MAX_ORDER_LIMIT],
            last_order_denominator: 0,
            last_order_numerator: 0,
            plan_orders_cur: 0,
            place_orders_cur: 0,
            valid_buy_order_num: 0,
            valid_sell_order_num: 0,
            padding3: [0; 10],
            free_slot_bits: std::u128::MAX,
        }
    }
}

impl_loadable!(TargetOrders);

impl TargetOrders {
    /// init
    #[inline]
    pub fn check_init(
        &mut self,
        x: u128,
        y: u128,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        if identity(self.owner) != Pubkey::default().to_aligned_bytes() {
            return Err(AmmError::AlreadyInUse.into());
        }
        self.owner = owner.to_aligned_bytes();
        self.last_order_numerator = 0; // 3
        self.last_order_denominator = 0; // 1

        self.plan_orders_cur = 0;
        self.place_orders_cur = 0;

        self.valid_buy_order_num = 0;
        self.valid_sell_order_num = 0;

        self.target_x = 0;
        self.target_y = 0;
        self.plan_x_buy = 0;
        self.plan_y_buy = 0;
        self.plan_x_sell = 0;
        self.plan_y_sell = 0;
        self.placed_x = 0;
        self.placed_y = 0;
        self.calc_pnl_x = x;
        self.calc_pnl_y = y;
        self.free_slot_bits = std::u128::MAX;
        Ok(())
    }

    /// load_mut_checked
    #[inline]
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
        owner: &Pubkey,
    ) -> Result<RefMut<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidTargetAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load_mut(account)?;
        if identity(data.owner) != owner.to_aligned_bytes() {
            return Err(AmmError::InvalidTargetOwner.into());
        }
        Ok(data)
    }

    /// load_checked
    #[inline]
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Ref<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidTargetAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load(account)?;
        if identity(data.owner) != owner.to_aligned_bytes() {
            return Err(AmmError::InvalidTargetOwner.into());
        }
        Ok(data)
    }
}

#[repr(u64)]
pub enum AmmStatus {
    Uninitialized = 0u64,
    Initialized = 1u64,
    Disabled = 2u64,
    WithdrawOnly = 3u64,
    // pool only can add or remove liquidity, can't swap and plan orders
    LiquidityOnly = 4u64,
    // pool only can add or remove liquidity and plan orders, can't swap
    OrderBookOnly = 5u64,
    // pool only can add or remove liquidity and swap, can't plan orders
    SwapOnly = 6u64,
    // pool status after created and will auto update to SwapOnly during swap after open_time
    WaitingTrade = 7u64,
}
impl AmmStatus {
    pub fn from_u64(status: u64) -> Self {
        match status {
            0u64 => AmmStatus::Uninitialized,
            1u64 => AmmStatus::Initialized,
            2u64 => AmmStatus::Disabled,
            3u64 => AmmStatus::WithdrawOnly,
            4u64 => AmmStatus::LiquidityOnly,
            5u64 => AmmStatus::OrderBookOnly,
            6u64 => AmmStatus::SwapOnly,
            7u64 => AmmStatus::WaitingTrade,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            AmmStatus::Uninitialized => 0u64,
            AmmStatus::Initialized => 1u64,
            AmmStatus::Disabled => 2u64,
            AmmStatus::WithdrawOnly => 3u64,
            AmmStatus::LiquidityOnly => 4u64,
            AmmStatus::OrderBookOnly => 5u64,
            AmmStatus::SwapOnly => 6u64,
            AmmStatus::WaitingTrade => 7u64,
        }
    }
    pub fn valid_status(status: u64) -> bool {
        match status {
            1u64 | 2u64 | 3u64 | 4u64 | 5u64 | 6u64 | 7u64 => return true,
            _ => return false,
        }
    }

    pub fn deposit_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn withdraw_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => true,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn swap_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => false,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn orderbook_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => false,
            AmmStatus::WaitingTrade => false,
        }
    }
}

#[repr(u64)]
pub enum AmmState {
    InvlidState = 0u64,
    IdleState = 1u64,
    CancelAllOrdersState = 2u64,
    PlanOrdersState = 3u64,
    CancelOrderState = 4u64,
    PlaceOrdersState = 5u64,
    PurgeOrderState = 6u64,
}
impl AmmState {
    pub fn from_u64(state: u64) -> Self {
        match state {
            0u64 => AmmState::InvlidState,
            1u64 => AmmState::IdleState,
            2u64 => AmmState::CancelAllOrdersState,
            3u64 => AmmState::PlanOrdersState,
            4u64 => AmmState::CancelOrderState,
            5u64 => AmmState::PlaceOrdersState,
            6u64 => AmmState::PurgeOrderState,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            AmmState::InvlidState => 0u64,
            AmmState::IdleState => 1u64,
            AmmState::CancelAllOrdersState => 2u64,
            AmmState::PlanOrdersState => 3u64,
            AmmState::CancelOrderState => 4u64,
            AmmState::PlaceOrdersState => 5u64,
            AmmState::PurgeOrderState => 6u64,
        }
    }
    pub fn valid_state(state: u64) -> bool {
        match state {
            0u64 | 1u64 | 2u64 | 3u64 | 4u64 | 5u64 | 6u64 => return true,
            _ => return false,
        }
    }
}

#[cfg_attr(feature = "client", derive(Debug))]
#[derive(Copy, Clone)]
#[repr(u64)]
pub enum AmmParams {
    Status = 0u64,
    State = 1u64,
    OrderNum = 2u64,
    Depth = 3u64,
    AmountWave = 4u64,
    MinPriceMultiplier = 5u64,
    MaxPriceMultiplier = 6u64,
    MinSize = 7u64,
    VolMaxCutRatio = 8u64,
    Fees = 9u64,
    AmmOwner = 10u64,
    SetOpenTime = 11u64,
    LastOrderDistance = 12u64,
    InitOrderDepth = 13u64,
    SetSwitchTime = 14u64,
    ClearOpenTime = 15u64,
    Seperate = 16u64,
    UpdateOpenOrder = 17u64,
}
impl AmmParams {
    pub fn from_u64(state: u64) -> Self {
        match state {
            0u64 => AmmParams::Status,
            1u64 => AmmParams::State,
            2u64 => AmmParams::OrderNum,
            3u64 => AmmParams::Depth,
            4u64 => AmmParams::AmountWave,
            5u64 => AmmParams::MinPriceMultiplier,
            6u64 => AmmParams::MaxPriceMultiplier,
            7u64 => AmmParams::MinSize,
            8u64 => AmmParams::VolMaxCutRatio,
            9u64 => AmmParams::Fees,
            10u64 => AmmParams::AmmOwner,
            11u64 => AmmParams::SetOpenTime,
            12u64 => AmmParams::LastOrderDistance,
            13u64 => AmmParams::InitOrderDepth,
            14u64 => AmmParams::SetSwitchTime,
            15u64 => AmmParams::ClearOpenTime,
            16u64 => AmmParams::Seperate,
            17u64 => AmmParams::UpdateOpenOrder,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            AmmParams::Status => 0u64,
            AmmParams::State => 1u64,
            AmmParams::OrderNum => 2u64,
            AmmParams::Depth => 3u64,
            AmmParams::AmountWave => 4u64,
            AmmParams::MinPriceMultiplier => 5u64,
            AmmParams::MaxPriceMultiplier => 6u64,
            AmmParams::MinSize => 7u64,
            AmmParams::VolMaxCutRatio => 8u64,
            AmmParams::Fees => 9u64,
            AmmParams::AmmOwner => 10u64,
            AmmParams::SetOpenTime => 11u64,
            AmmParams::LastOrderDistance => 12u64,
            AmmParams::InitOrderDepth => 13u64,
            AmmParams::SetSwitchTime => 14u64,
            AmmParams::ClearOpenTime => 15u64,
            AmmParams::Seperate => 16u64,
            AmmParams::UpdateOpenOrder => 17u64,
        }
    }
}

#[cfg_attr(feature = "client", derive(Debug))]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u64)]
pub enum AmmResetFlag {
    ResetYes = 0u64,
    ResetNo = 1u64,
}
impl AmmResetFlag {
    pub fn from_u64(flag: u64) -> Self {
        match flag {
            0u64 => AmmResetFlag::ResetYes,
            1u64 => AmmResetFlag::ResetNo,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            AmmResetFlag::ResetYes => 0u64,
            AmmResetFlag::ResetNo => 1u64,
        }
    }
}

fn validate_fraction(
    numerator: u64,
    denominator: u64,
) -> Result<(), AmmError> {
    if numerator >= denominator || denominator == 0 {
        Err(AmmError::InvalidFee)
    } else {
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fees {
    /// numerator of the min_separate
    pub min_separate_numerator: u64,
    /// denominator of the min_separate
    pub min_separate_denominator: u64,

    /// numerator of the fee
    pub trade_fee_numerator: u64,
    /// denominator of the fee
    /// and 'trade_fee_denominator' must be equal to 'min_separate_denominator'
    pub trade_fee_denominator: u64,

    /// numerator of the pnl
    pub pnl_numerator: u64,
    /// denominator of the pnl
    pub pnl_denominator: u64,

    /// numerator of the swap_fee
    pub swap_fee_numerator: u64,
    /// denominator of the swap_fee
    pub swap_fee_denominator: u64,
}

impl Fees {
    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<(), AmmError> {
        validate_fraction(
            self.min_separate_numerator,
            self.min_separate_denominator,
        )?;
        validate_fraction(
            self.trade_fee_numerator,
            self.trade_fee_denominator,
        )?;
        validate_fraction(self.pnl_numerator, self.pnl_denominator)?;
        validate_fraction(self.swap_fee_numerator, self.swap_fee_denominator)?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<(), AmmError> {
        // min_separate = 5/10000
        self.min_separate_numerator = 5;
        self.min_separate_denominator = TEN_THOUSAND;
        // trade_fee = 25/10000
        self.trade_fee_numerator = 25;
        self.trade_fee_denominator = TEN_THOUSAND;
        // pnl = 12/100
        self.pnl_numerator = 12;
        self.pnl_denominator = 100;
        // swap_fee = 25 / 10000
        self.swap_fee_numerator = 25;
        self.swap_fee_denominator = TEN_THOUSAND;
        Ok(())
    }
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for Fees {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for Fees {}
impl Pack for Fees {
    const LEN: usize = 64;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 64];
        let (
            min_separate_numerator,
            min_separate_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            pnl_numerator,
            pnl_denominator,
            swap_fee_numerator,
            swap_fee_denominator,
        ) = mut_array_refs![output, 8, 8, 8, 8, 8, 8, 8, 8];
        *min_separate_numerator = self.min_separate_numerator.to_le_bytes();
        *min_separate_denominator =
            self.min_separate_denominator.to_le_bytes();
        *trade_fee_numerator = self.trade_fee_numerator.to_le_bytes();
        *trade_fee_denominator = self.trade_fee_denominator.to_le_bytes();
        *pnl_numerator = self.pnl_numerator.to_le_bytes();
        *pnl_denominator = self.pnl_denominator.to_le_bytes();
        *swap_fee_numerator = self.swap_fee_numerator.to_le_bytes();
        *swap_fee_denominator = self.swap_fee_denominator.to_le_bytes();
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Fees, ProgramError> {
        let input = array_ref![input, 0, 64];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            min_separate_numerator,
            min_separate_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            pnl_numerator,
            pnl_denominator,
            swap_fee_numerator,
            swap_fee_denominator,
        ) = array_refs![input, 8, 8, 8, 8, 8, 8, 8, 8];
        Ok(Self {
            min_separate_numerator: u64::from_le_bytes(
                *min_separate_numerator,
            ),
            min_separate_denominator: u64::from_le_bytes(
                *min_separate_denominator,
            ),
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            pnl_numerator: u64::from_le_bytes(*pnl_numerator),
            pnl_denominator: u64::from_le_bytes(*pnl_denominator),
            swap_fee_numerator: u64::from_le_bytes(*swap_fee_numerator),
            swap_fee_denominator: u64::from_le_bytes(*swap_fee_denominator),
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// padding for future updates
    // pub padding: [u64; 2],
    // based on TypeScript LIQUIDITY_STATE_LAYOUT_V4
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    /// switch from orderbookonly to init
    pub orderbook_to_init_time: u64,

    /// swap coin in amount
    pub swap_coin_in_amount: u128,
    /// swap pc out amount
    pub swap_pc_out_amount: u128,
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,

    /// swap pc in amount
    pub swap_pc_in_amount: u64,
    pub padding_some: [u64; 1],
    /// swap coin out amount
    pub swap_coin_out_amount: u64,
    pub padding_some_2: [u64; 1],
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}

impl StateData {
    pub fn initialize(&mut self, open_time: u64) -> Result<(), AmmError> {
        self.need_take_pnl_coin = 0u64;
        self.need_take_pnl_pc = 0u64;
        self.total_pnl_pc = 0u64;
        self.total_pnl_coin = 0u64;
        self.pool_open_time = open_time;
        self.punish_pc_amount = 0u64;
        self.punish_coin_amount = 0u64;
        self.orderbook_to_init_time = 0u64;
        self.swap_coin_in_amount = 0u128;
        self.swap_pc_out_amount = 0u128;
        self.swap_acc_pc_fee = 0u64;
        self.swap_pc_in_amount = 0u64;
        self.swap_coin_out_amount = 0u64;
        self.swap_acc_coin_fee = 0u64;

        Ok(())
    }
}

impl_loadable!(AmmInfo);

#[cfg_attr(feature = "client", derive(Debug))]
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    /// The program address is created deterministically with the nonce,
    /// amm program id, and amm account pubkey.  This program address has
    /// authority over the amm's token coin account, token pc account, and pool
    /// token mint.
    pub nonce: u64,
    /// max order count
    pub order_num: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    // pub padding1: [u64; 8],
    // instead of padding (based on TypeScript LIQUIDITY_STATE_LAYOUT_V4)
    pub withdraw_queue: Pubkey,
    pub lp_vault: Pubkey,
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,

    pub client_order_id: u64,

    /// padding
    pub padding: [u64; 2],
}

impl AmmInfo {
    /// Helper function to get the more efficient packed size of the struct
    /// load_mut_checked
    #[inline]
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
    ) -> Result<RefMut<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidAmmAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load_mut(account)?;
        if data.status == AmmStatus::Uninitialized as u64 {
            return Err(AmmError::InvalidStatus.into());
        }
        Ok(data)
    }

    /// load_checked
    #[inline]
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
    ) -> Result<Ref<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidAmmAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load(account)?;
        if data.status == AmmStatus::Uninitialized as u64 {
            return Err(AmmError::InvalidStatus.into());
        }
        Ok(data)
    }

    pub fn initialize(
        &mut self,
        nonce: u8,
        open_time: u64,
        coin_decimals: u8,
        pc_decimals: u8,
        coin_lot_size: u64,
        pc_lot_size: u64,
    ) -> Result<(), AmmError> {
        self.fees.initialize()?;
        self.state_data.initialize(open_time)?;

        self.status = AmmStatus::Uninitialized.into_u64();
        self.nonce = nonce as u64;
        self.order_num = 7;
        self.depth = 3;
        self.coin_decimals = coin_decimals as u64;
        self.pc_decimals = pc_decimals as u64;
        self.state = AmmState::IdleState.into_u64();
        self.reset_flag = AmmResetFlag::ResetNo.into_u64();
        if pc_decimals > coin_decimals {
            self.sys_decimal_value = (10 as u64)
                .checked_pow(pc_decimals.try_into().unwrap())
                .unwrap();
        } else {
            self.sys_decimal_value = (10 as u64)
                .checked_pow(coin_decimals.try_into().unwrap())
                .unwrap();
        }
        let temp_value_numerator = (coin_lot_size as u128)
            .checked_mul(
                (10 as u128)
                    .checked_pow(pc_decimals.try_into().unwrap())
                    .unwrap(),
            )
            .unwrap();
        let temp_value_denominator = (pc_lot_size as u128)
            .checked_mul(
                (10 as u128)
                    .checked_pow(coin_decimals.try_into().unwrap())
                    .unwrap(),
            )
            .unwrap();
        if (self.sys_decimal_value as u128)
            <= temp_value_numerator
                .checked_div(temp_value_denominator)
                .unwrap()
        {
            self.sys_decimal_value = Calculator::to_u64(
                temp_value_numerator
                    .checked_div(temp_value_denominator)
                    .unwrap(),
            )
            .unwrap();
        }
        let min_size = (coin_lot_size as u128)
            .checked_mul(self.sys_decimal_value as u128)
            .unwrap()
            .checked_div(
                (10u128)
                    .checked_pow(coin_decimals.try_into().unwrap())
                    .unwrap(),
            )
            .unwrap();
        if min_size < u64::max_value().into() {
            self.min_size = Calculator::to_u64(min_size)?;
        } else {
            // must check not zero in process_monitor_step
            self.min_size = 0;
        }
        self.vol_max_cut_ratio = 500; // TEN_THOUSAND as denominator
        self.amount_wave = self
            .sys_decimal_value
            .checked_mul(5)
            .unwrap()
            .checked_div(1000)
            .unwrap();
        self.coin_lot_size = coin_lot_size;
        self.pc_lot_size = Calculator::convert_in_pc_lot_size(
            pc_decimals,
            coin_decimals,
            pc_lot_size,
            coin_lot_size,
            self.sys_decimal_value,
        );
        self.min_price_multiplier = 1;
        self.max_price_multiplier = 1000000000;
        self.padding = Zeroable::zeroed();

        Ok(())
    }

    pub fn incr_client_order_id(&mut self) -> u64 {
        self.client_order_id = self.client_order_id.wrapping_add(1);
        if self.client_order_id == 0 {
            self.client_order_id += 1;
        }
        self.client_order_id
    }
}

impl_loadable!(AmmConfig);
/// State of amm config account
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmConfig {
    /// withdraw pnl owner
    pub pnl_owner: Pubkey,
    /// admin amm order owner
    pub cancel_owner: Pubkey,
    /// pending
    pub pending_1: [u64; 28],
    /// pending
    pub pending_2: [u64; 31],
    /// init amm pool fee amount
    pub create_pool_fee: u64,
}

impl AmmConfig {
    /// Helper function to get the more efficient packed size of the struct
    /// load_mut_checked
    #[inline]
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
    ) -> Result<RefMut<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load_mut(account)?;
        Ok(data)
    }

    /// load_checked
    #[inline]
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
    ) -> Result<Ref<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load(account)?;
        Ok(data)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LastOrderDistance {
    pub last_order_numerator: u64,
    pub last_order_denominator: u64,
}

/// For simulateTransaction to get instruction data
#[cfg_attr(feature = "client", derive(Debug))]
#[derive(Copy, Clone)]
#[repr(u64)]
pub enum SimulateParams {
    PoolInfo = 0u64,
    SwapBaseInInfo = 1u64,
    SwapBaseOutInfo = 2u64,
    RunCrankInfo = 3u64,
}
impl SimulateParams {
    pub fn from_u64(flag: u64) -> Self {
        match flag {
            0u64 => SimulateParams::PoolInfo,
            1u64 => SimulateParams::SwapBaseInInfo,
            2u64 => SimulateParams::SwapBaseOutInfo,
            3u64 => SimulateParams::RunCrankInfo,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            SimulateParams::PoolInfo => 0u64,
            SimulateParams::SwapBaseInInfo => 1u64,
            SimulateParams::SwapBaseOutInfo => 2u64,
            SimulateParams::RunCrankInfo => 3u64,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunCrankData {
    pub status: u64,
    pub state: u64,
    pub run_crank: bool,
}
impl RunCrankData {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn from_json(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetPoolData {
    pub status: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub lp_decimals: u64,
    // pool token vault without pnl
    pub pool_pc_amount: u64,
    pub pool_coin_amount: u64,
    pub pnl_pc_amount: u64,
    pub pnl_coin_amount: u64,
    pub pool_lp_supply: u64,
    pub pool_open_time: u64,
    pub amm_id: String,
}
impl GetPoolData {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn from_json(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetSwapBaseInData {
    pub pool_data: GetPoolData,
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub price_impact: u64,
}
impl GetSwapBaseInData {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn from_json(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetSwapBaseOutData {
    pub pool_data: GetPoolData,
    pub max_amount_in: u64,
    pub amount_out: u64,
    pub price_impact: u64,
}
impl GetSwapBaseOutData {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn from_json(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}
