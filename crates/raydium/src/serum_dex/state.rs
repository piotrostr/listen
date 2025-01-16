#![cfg_attr(not(feature = "program"), allow(unused))]
use num_enum::TryFromPrimitive;
use std::{
    cell::{Ref, RefMut},
    convert::identity,
    convert::TryInto,
    mem::size_of,
    num::NonZeroU64,
    ops::Deref,
    ops::DerefMut,
};

use arrayref::{array_ref, array_refs, mut_array_refs};

use bytemuck::{
    bytes_of, bytes_of_mut, cast, cast_slice, cast_slice_mut, from_bytes_mut, try_cast_mut,
    try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut, Pod, Zeroable,
};
use enumflags2::BitFlags;
use num_traits::FromPrimitive;
use safe_transmute::{self, to_bytes::transmute_to_bytes, trivial::TriviallyTransmutable};

use solana_sdk::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    rent::Rent, sysvar::Sysvar,
};
use spl_token::error::TokenError;

use crate::serum_dex::{
    critbit::Slab,
    error::{DexError, DexErrorCode, DexResult, SourceFileId},
    fees::{self, FeeTier},
    instruction::{
        disable_authority, fee_sweeper, msrm_token, srm_token, CancelOrderInstructionV2,
        InitializeMarketInstruction, MarketInstruction, NewOrderInstructionV3, SelfTradeBehavior,
        SendTakeInstruction,
    },
    matching::{OrderBookState, OrderType, RequestProceeds, Side},
};

use anchor_lang::prelude::{borsh, emit, event, AnchorDeserialize, AnchorSerialize};

declare_check_assert_macros!(SourceFileId::State);

pub trait ToAlignedBytes {
    fn to_aligned_bytes(&self) -> [u64; 4];
}

impl ToAlignedBytes for Pubkey {
    #[inline]
    fn to_aligned_bytes(&self) -> [u64; 4] {
        cast(self.to_bytes())
    }
}

#[derive(Copy, Clone, BitFlags, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum AccountFlag {
    Initialized = 1u64 << 0,
    Market = 1u64 << 1,
    OpenOrders = 1u64 << 2,
    RequestQueue = 1u64 << 3,
    EventQueue = 1u64 << 4,
    Bids = 1u64 << 5,
    Asks = 1u64 << 6,
    Disabled = 1u64 << 7,
    Closed = 1u64 << 8,
    Permissioned = 1u64 << 9,
    CrankAuthorityRequired = 1u64 << 10,
}

// Versioned frontend for market accounts.
pub enum Market<'a> {
    V1(RefMut<'a, MarketState>),
    V2(RefMut<'a, MarketStateV2>),
    V1Ref(Ref<'a, MarketState>),
    V2Ref(Ref<'a, MarketStateV2>),
}

impl<'a> Deref for Market<'a> {
    type Target = MarketState;

    fn deref(&self) -> &Self::Target {
        match self {
            Market::V1(v1) => v1.deref(),
            Market::V2(v2) => v2.deref(),
            Market::V1Ref(v1_ref) => v1_ref.deref(),
            Market::V2Ref(v2_ref) => v2_ref.deref(),
        }
    }
}

impl<'a> DerefMut for Market<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Market::V1(v1) => v1.deref_mut(),
            Market::V2(v2) => v2.deref_mut(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Market<'a> {
    #[inline]
    pub fn load(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        // Allow for the market flag to be set to AccountFlag::Disabled
        allow_disabled: bool,
    ) -> DexResult<Self> {
        let flags = Market::account_flags(&market_account.try_borrow_data()?)?;
        if flags.intersects(AccountFlag::Permissioned) {
            Ok(Market::V2(MarketStateV2::load(
                market_account,
                program_id,
                allow_disabled,
            )?))
        } else {
            Ok(Market::V1(MarketState::load(
                market_account,
                program_id,
                allow_disabled,
            )?))
        }
    }

    #[inline]
    pub fn load_checked(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        // Allow for the market flag to be set to AccountFlag::Disabled
        allow_disabled: bool,
    ) -> DexResult<Self> {
        let flags = Market::account_flags(&market_account.try_borrow_data()?)?;
        if flags.intersects(AccountFlag::Permissioned) {
            Ok(Market::V2Ref(MarketStateV2::load_checked(
                market_account,
                program_id,
                allow_disabled,
            )?))
        } else {
            Ok(Market::V1Ref(MarketState::load_checked(
                market_account,
                program_id,
                allow_disabled,
            )?))
        }
    }

    pub fn account_flags(account_data: &[u8]) -> DexResult<BitFlags<AccountFlag>> {
        let start = ACCOUNT_HEAD_PADDING.len();
        let end = start + size_of::<AccountFlag>();
        check_assert!(account_data.len() >= end)?;

        let mut flag_bytes = [0u8; 8];
        flag_bytes.copy_from_slice(&account_data[start..end]);

        BitFlags::from_bits(u64::from_le_bytes(flag_bytes))
            .map_err(|_| DexErrorCode::InvalidMarketFlags.into())
            .map(Into::into)
    }

    pub fn open_orders_authority(&self) -> Option<&Pubkey> {
        match &self {
            Market::V1(_) | Market::V1Ref(_) => None,
            Market::V2(state) => Some(&state.open_orders_authority),
            Market::V2Ref(state) => Some(&state.open_orders_authority),
        }
    }

    pub fn prune_authority(&self) -> Option<&Pubkey> {
        match &self {
            Market::V1(_) | Market::V1Ref(_) => None,
            Market::V2(state) => Some(&state.prune_authority),
            Market::V2Ref(state) => Some(&state.prune_authority),
        }
    }

    pub fn consume_events_authority(&self) -> Option<&Pubkey> {
        match &self {
            Market::V1(_) | Market::V1Ref(_) => None,
            Market::V2(state) => {
                let flags = BitFlags::from_bits(state.inner.account_flags).unwrap();
                if flags.intersects(AccountFlag::CrankAuthorityRequired) {
                    Some(&state.consume_events_authority)
                } else {
                    None
                }
            }
            Market::V2Ref(state) => {
                let flags = BitFlags::from_bits(state.inner.account_flags).unwrap();
                if flags.intersects(AccountFlag::CrankAuthorityRequired) {
                    Some(&state.consume_events_authority)
                } else {
                    None
                }
            }
        }
    }

    pub fn load_orders_mut(
        &self,
        orders_account: &'a AccountInfo,
        owner_account: Option<&AccountInfo>,
        program_id: &Pubkey,
        rent: Option<Rent>,
        open_orders_authority: Option<account_parser::SignerAccount>,
    ) -> DexResult<RefMut<'a, OpenOrders>> {
        check_assert_eq!(orders_account.owner, program_id)?;
        let mut open_orders: RefMut<'a, OpenOrders>;

        let open_orders_data_len = orders_account.data_len();
        let open_orders_lamports = orders_account.lamports();
        let (_, data) = strip_header::<[u8; 0], u8>(orders_account, true)?;
        open_orders = RefMut::map(data, |data| from_bytes_mut(data));

        if open_orders.account_flags == 0 {
            let oo_authority = open_orders_authority.map(|a| a.inner().key);
            if oo_authority != self.open_orders_authority() {
                return Err(DexErrorCode::InvalidOpenOrdersAuthority.into());
            }

            let rent = rent.ok_or(DexErrorCode::RentNotProvided)?;
            let owner_account = owner_account.ok_or(DexErrorCode::OwnerAccountNotProvided)?;
            if !rent.is_exempt(open_orders_lamports, open_orders_data_len) {
                return Err(DexErrorCode::OrdersNotRentExempt)?;
            }
            open_orders.init(
                &identity(self.own_address),
                &owner_account.key.to_aligned_bytes(),
            )?;
        }

        open_orders.check_flags()?;
        check_assert_eq!(identity(open_orders.market), identity(self.own_address))
            .map_err(|_| DexErrorCode::WrongOrdersAccount)?;
        if let Some(owner) = owner_account {
            check_assert_eq!(&identity(open_orders.owner), &owner.key.to_aligned_bytes())
                .map_err(|_| DexErrorCode::WrongOrdersAccount)?;
        }

        Ok(open_orders)
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(target_endian = "little", derive(Debug))]
#[repr(packed)]
pub struct MarketStateV2 {
    pub inner: MarketState,
    pub open_orders_authority: Pubkey,
    pub prune_authority: Pubkey,
    pub consume_events_authority: Pubkey,
    // Unused bytes for future upgrades.
    padding: [u8; 992],
}

impl Deref for MarketStateV2 {
    type Target = MarketState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MarketStateV2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl MarketStateV2 {
    #[inline]
    pub fn load<'a>(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        allow_disabled: bool,
    ) -> DexResult<RefMut<'a, Self>> {
        check_assert_eq!(market_account.owner, program_id)?;

        let mut account_data: RefMut<'a, [u8]>;
        let state: RefMut<'a, Self>;

        account_data = RefMut::map(market_account.try_borrow_mut_data()?, |data| *data);
        check_account_padding(&mut account_data)?;

        state = RefMut::map(account_data, |data| {
            from_bytes_mut(cast_slice_mut(
                check_account_padding(data).unwrap_or_else(|_| unreachable!()),
            ))
        });

        state.check_flags(allow_disabled)?;
        Ok(state)
    }

    #[inline]
    pub fn load_checked<'a>(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        allow_disabled: bool,
    ) -> DexResult<Ref<'a, Self>> {
        check_assert_eq!(market_account.owner, program_id)?;

        let account_data = market_account.try_borrow_data()?;
        check_assert!(account_data.len() >= 12)?;
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        check_assert_eq!(head, ACCOUNT_HEAD_PADDING)?;
        check_assert_eq!(tail, ACCOUNT_TAIL_PADDING)?;

        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| {
            bytemuck::from_bytes(&account_data[5..account_data.len() - 7])
        });

        state.check_flags(allow_disabled)?;
        Ok(state)
    }

    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> DexResult {
        let flags = BitFlags::from_bits(self.account_flags)
            .map_err(|_| DexErrorCode::InvalidMarketFlags)?;

        let required_flags =
            AccountFlag::Initialized | AccountFlag::Market | AccountFlag::Permissioned;
        let required_crank_flags = required_flags | AccountFlag::CrankAuthorityRequired;

        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            let disabled_crank_flags = required_crank_flags | AccountFlag::Disabled;
            if flags != required_flags
                && flags != required_crank_flags
                && flags != disabled_flags
                && flags != disabled_crank_flags
            {
                return Err(DexErrorCode::InvalidMarketFlags.into());
            }
        } else {
            if flags != required_flags && flags != required_crank_flags {
                return Err(DexErrorCode::InvalidMarketFlags.into());
            }
        }

        Ok(())
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for MarketStateV2 {}

#[cfg(target_endian = "little")]
unsafe impl Pod for MarketStateV2 {}

#[cfg(target_endian = "little")]
unsafe impl TriviallyTransmutable for MarketStateV2 {}

#[derive(Copy, Clone)]
#[cfg_attr(target_endian = "little", derive(Debug))]
#[repr(packed)]
pub struct MarketState {
    // 0
    pub account_flags: u64, // Initialized, Market

    // 1
    pub own_address: [u64; 4],

    // 5
    pub vault_signer_nonce: u64,
    // 6
    pub coin_mint: [u64; 4],
    // 10
    pub pc_mint: [u64; 4],

    // 14
    pub coin_vault: [u64; 4],
    // 18
    pub coin_deposits_total: u64,
    // 19
    pub coin_fees_accrued: u64,

    // 20
    pub pc_vault: [u64; 4],
    // 24
    pub pc_deposits_total: u64,
    // 25
    pub pc_fees_accrued: u64,

    // 26
    pub pc_dust_threshold: u64,

    // 27
    pub req_q: [u64; 4],
    // 31
    pub event_q: [u64; 4],

    // 35
    pub bids: [u64; 4],
    // 39
    pub asks: [u64; 4],

    // 43
    pub coin_lot_size: u64,
    // 44
    pub pc_lot_size: u64,

    // 45
    pub fee_rate_bps: u64,
    // 46
    pub referrer_rebates_accrued: u64,
}
#[cfg(target_endian = "little")]
unsafe impl Zeroable for MarketState {}
#[cfg(target_endian = "little")]
unsafe impl Pod for MarketState {}
#[cfg(target_endian = "little")]
unsafe impl TriviallyTransmutable for MarketState {}

pub const ACCOUNT_HEAD_PADDING: &[u8; 5] = b"serum";
pub const ACCOUNT_TAIL_PADDING: &[u8; 7] = b"padding";

fn init_account_padding(data: &mut [u8]) -> DexResult<&mut [[u8; 8]]> {
    check_assert!(data.len() >= 12)?;
    let (head, data, tail) = mut_array_refs![data, 5; ..; 7];
    *head = *ACCOUNT_HEAD_PADDING;
    *tail = *ACCOUNT_TAIL_PADDING;
    Ok(try_cast_slice_mut(data).or(check_unreachable!())?)
}

fn check_account_padding(data: &mut [u8]) -> DexResult<&mut [[u8; 8]]> {
    check_assert!(data.len() >= 12)?;
    let (head, data, tail) = mut_array_refs![data, 5; ..; 7];
    check_assert_eq!(head, ACCOUNT_HEAD_PADDING)?;
    check_assert_eq!(tail, ACCOUNT_TAIL_PADDING)?;
    Ok(try_cast_slice_mut(data).or(check_unreachable!())?)
}

fn check_account_padding_checked(account_data: &[u8]) -> DexResult<&[[u8; 8]]> {
    check_assert!(account_data.len() >= 12)?;
    let head = array_ref![account_data, 0, 5];
    let tail = array_ref![account_data, account_data.len() - 7, 7];
    check_assert_eq!(head, ACCOUNT_HEAD_PADDING)?;
    check_assert_eq!(tail, ACCOUNT_TAIL_PADDING)?;
    let data = &account_data[5..account_data.len() - 7];
    Ok(try_cast_slice(&data).or(check_unreachable!())?)
}

fn strip_account_padding(padded_data: &mut [u8], init_allowed: bool) -> DexResult<&mut [[u8; 8]]> {
    if init_allowed {
        init_account_padding(padded_data)
    } else {
        check_account_padding(padded_data)
    }
}

fn strip_account_padding_checked(padded_data: &[u8]) -> DexResult<&[[u8; 8]]> {
    check_account_padding_checked(padded_data)
}

pub fn strip_header<'a, H: Pod, D: Pod>(
    account: &'a AccountInfo,
    init_allowed: bool,
) -> DexResult<(RefMut<'a, H>, RefMut<'a, [D]>)> {
    let mut result = Ok(());
    let (header, inner): (RefMut<'a, [H]>, RefMut<'a, [D]>) =
        RefMut::map_split(account.try_borrow_mut_data()?, |padded_data| {
            let dummy_value: (&mut [H], &mut [D]) = (&mut [], &mut []);
            let padded_data: &mut [u8] = *padded_data;
            let u64_data = match strip_account_padding(padded_data, init_allowed) {
                Ok(u64_data) => u64_data,
                Err(e) => {
                    result = Err(e);
                    return dummy_value;
                }
            };

            let data: &mut [u8] = cast_slice_mut(u64_data);
            let (header_bytes, inner_bytes) = data.split_at_mut(size_of::<H>());
            let header: &mut H;
            let inner: &mut [D];

            header = match try_from_bytes_mut(header_bytes) {
                Ok(h) => h,
                Err(_e) => {
                    result = Err(assertion_error!().into());
                    return dummy_value;
                }
            };
            inner = remove_slop_mut(inner_bytes);

            (std::slice::from_mut(header), inner)
        });
    result?;
    let header = RefMut::map(header, |s| s.first_mut().unwrap_or_else(|| unreachable!()));
    Ok((header, inner))
}

pub fn strip_header_checked<'a, H: Pod, D: Pod>(
    account: &'a AccountInfo,
) -> DexResult<(Ref<'a, H>, Ref<'a, [D]>)> {
    let mut result = Ok(());
    let (header, inner): (Ref<'a, [H]>, Ref<'a, [D]>) =
        Ref::map_split(account.try_borrow_data()?, |padded_data| {
            let dummy_value: (&[H], &[D]) = (&[], &[]);
            let padded_data: &[u8] = *padded_data;
            let u64_data = match strip_account_padding_checked(padded_data) {
                Ok(u64_data) => u64_data,
                Err(e) => {
                    result = Err(e);
                    return dummy_value;
                }
            };

            let data: &[u8] = cast_slice(u64_data);
            let (header_bytes, inner_bytes) = data.split_at(size_of::<H>());
            let header: &H;
            let inner: &[D];

            header = match try_from_bytes(header_bytes) {
                Ok(h) => h,
                Err(_e) => {
                    result = Err(assertion_error!().into());
                    return dummy_value;
                }
            };
            inner = remove_slop(inner_bytes);

            (std::slice::from_ref(header), inner)
        });
    result?;
    let header = Ref::map(header, |s| s.first().unwrap_or_else(|| unreachable!()));
    Ok((header, inner))
}

impl MarketState {
    #[inline]
    pub fn load<'a>(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        allow_disabled: bool,
    ) -> DexResult<RefMut<'a, Self>> {
        check_assert_eq!(market_account.owner, program_id)?;
        let mut account_data: RefMut<'a, [u8]>;
        let state: RefMut<'a, Self>;

        account_data = RefMut::map(market_account.try_borrow_mut_data()?, |data| *data);
        check_account_padding(&mut account_data)?;
        state = RefMut::map(account_data, |data| {
            from_bytes_mut(cast_slice_mut(
                check_account_padding(data).unwrap_or_else(|_| unreachable!()),
            ))
        });

        state.check_flags(allow_disabled)?;
        Ok(state)
    }

    #[inline]
    pub fn load_checked<'a>(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        allow_disabled: bool,
    ) -> DexResult<Ref<'a, Self>> {
        check_assert_eq!(market_account.owner, program_id)?;

        let account_data = market_account.try_borrow_data()?;
        check_assert!(account_data.len() >= 12)?;
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        check_assert_eq!(head, ACCOUNT_HEAD_PADDING)?;
        check_assert_eq!(tail, ACCOUNT_TAIL_PADDING)?;

        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| {
            bytemuck::from_bytes(&account_data[5..account_data.len() - 7])
        });

        state.check_flags(allow_disabled)?;
        Ok(state)
    }

    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> DexResult {
        let flags = BitFlags::from_bits(self.account_flags)
            .map_err(|_| DexErrorCode::InvalidMarketFlags)?;
        let required_flags = AccountFlag::Initialized | AccountFlag::Market;
        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            if flags != required_flags && flags != disabled_flags {
                return Err(DexErrorCode::InvalidMarketFlags.into());
            }
        } else {
            if flags != required_flags {
                return Err(DexErrorCode::InvalidMarketFlags.into());
            }
        }
        Ok(())
    }

    pub fn load_bids_mut<'a>(&self, bids: &'a AccountInfo) -> DexResult<RefMut<'a, Slab>> {
        check_assert_eq!(&bids.key.to_aligned_bytes(), &identity(self.bids))
            .map_err(|_| DexErrorCode::WrongBidsAccount)?;
        let (header, buf) = strip_header::<OrderBookStateHeader, u8>(bids, false)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(&flags, &(AccountFlag::Initialized | AccountFlag::Bids))?;
        Ok(RefMut::map(buf, Slab::new))
    }

    pub fn load_bids_checked<'a>(&self, bids: &'a AccountInfo) -> DexResult<Ref<'a, Slab>> {
        check_assert_eq!(&bids.key.to_aligned_bytes(), &identity(self.bids))
            .map_err(|_| DexErrorCode::WrongBidsAccount)?;
        let (header, buf) = strip_header_checked::<OrderBookStateHeader, u8>(bids)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(&flags, &(AccountFlag::Initialized | AccountFlag::Bids))?;
        Ok(Ref::map(buf, Slab::new_check))
    }

    pub fn load_asks_mut<'a>(&self, asks: &'a AccountInfo) -> DexResult<RefMut<'a, Slab>> {
        check_assert_eq!(&asks.key.to_aligned_bytes(), &identity(self.asks))
            .map_err(|_| DexErrorCode::WrongAsksAccount)?;
        let (header, buf) = strip_header::<OrderBookStateHeader, u8>(asks, false)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(&flags, &(AccountFlag::Initialized | AccountFlag::Asks))?;
        Ok(RefMut::map(buf, Slab::new))
    }

    pub fn load_asks_checked<'a>(&self, asks: &'a AccountInfo) -> DexResult<Ref<'a, Slab>> {
        check_assert_eq!(&asks.key.to_aligned_bytes(), &identity(self.asks))
            .map_err(|_| DexErrorCode::WrongAsksAccount)?;
        let (header, buf) = strip_header_checked::<OrderBookStateHeader, u8>(asks)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(&flags, &(AccountFlag::Initialized | AccountFlag::Asks))?;
        Ok(Ref::map(buf, Slab::new_check))
    }

    fn load_request_queue_mut<'a>(&self, queue: &'a AccountInfo) -> DexResult<RequestQueue<'a>> {
        check_assert_eq!(&queue.key.to_aligned_bytes(), &identity(self.req_q))
            .map_err(|_| DexErrorCode::WrongRequestQueueAccount)?;

        let (header, buf) = strip_header::<RequestQueueHeader, Request>(queue, false)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(
            &flags,
            &(AccountFlag::Initialized | AccountFlag::RequestQueue)
        )?;
        Ok(Queue { header, buf })
    }

    pub fn load_event_queue_mut<'a>(&self, queue: &'a AccountInfo) -> DexResult<EventQueue<'a>> {
        check_assert_eq!(&queue.key.to_aligned_bytes(), &identity(self.event_q))
            .map_err(|_| DexErrorCode::WrongEventQueueAccount)?;
        let (header, buf) = strip_header::<EventQueueHeader, Event>(queue, false)?;

        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        check_assert_eq!(
            &flags,
            &(AccountFlag::Initialized | AccountFlag::EventQueue)
        )?;
        Ok(Queue { header, buf })
    }

    #[inline]
    fn check_coin_vault(&self, vault: account_parser::TokenAccount) -> DexResult {
        if identity(self.coin_vault) != vault.inner().key.to_aligned_bytes() {
            Err(DexErrorCode::WrongCoinVault)?
        }
        Ok(())
    }

    #[inline]
    fn check_pc_vault(&self, vault: account_parser::TokenAccount) -> DexResult {
        if identity(self.pc_vault) != vault.inner().key.to_aligned_bytes() {
            Err(DexErrorCode::WrongPcVault)?
        }
        Ok(())
    }

    #[inline]
    fn check_coin_payer(&self, payer: account_parser::TokenAccount) -> DexResult {
        if &payer.inner().try_borrow_data()?[..32] != transmute_to_bytes(&identity(self.coin_mint))
        {
            Err(DexErrorCode::WrongCoinMint)?
        }
        Ok(())
    }

    #[inline]
    fn check_pc_payer(&self, payer: account_parser::TokenAccount) -> DexResult {
        if &payer.inner().try_borrow_data()?[..32] != transmute_to_bytes(&identity(self.pc_mint)) {
            Err(DexErrorCode::WrongPcMint)?
        }
        Ok(())
    }

    #[inline]
    fn load_fee_tier(
        &self,
        expected_owner: &[u64; 4],
        srm_or_msrm_account: Option<account_parser::TokenAccount>,
    ) -> DexResult<FeeTier> {
        let market_addr = self.pubkey();
        let srm_or_msrm_account = match srm_or_msrm_account {
            Some(a) => a,
            None => return Ok(FeeTier::from_srm_and_msrm_balances(&market_addr, 0, 0)),
        };
        let data = srm_or_msrm_account.inner().try_borrow_data()?;

        let mut aligned_data: [u64; 9] = Zeroable::zeroed();
        bytes_of_mut(&mut aligned_data).copy_from_slice(&data[..72]);
        let (mint, owner, &[balance]) = array_refs![&aligned_data, 4, 4, 1];

        check_assert_eq!(owner, expected_owner)?;
        if mint == &srm_token::ID.to_aligned_bytes() {
            return Ok(FeeTier::from_srm_and_msrm_balances(
                &market_addr,
                balance,
                0,
            ));
        }

        if mint == &msrm_token::ID.to_aligned_bytes() {
            return Ok(FeeTier::from_srm_and_msrm_balances(
                &market_addr,
                0,
                balance,
            ));
        }

        Ok(FeeTier::from_srm_and_msrm_balances(&market_addr, 0, 0))
    }

    fn check_enabled(&self) -> DexResult {
        let flags = BitFlags::from_bits(self.account_flags).unwrap();
        if flags.contains(AccountFlag::Disabled) {
            return Err(DexErrorCode::MarketIsDisabled.into());
        }
        Ok(())
    }

    fn pubkey(&self) -> Pubkey {
        Pubkey::new(cast_slice(&identity(self.own_address) as &[_]))
    }
}

#[repr(packed)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "fuzz", derive(Debug))]
pub struct OpenOrders {
    pub account_flags: u64, // Initialized, OpenOrders
    pub market: [u64; 4],
    pub owner: [u64; 4],

    pub native_coin_free: u64,
    pub native_coin_total: u64,

    pub native_pc_free: u64,
    pub native_pc_total: u64,

    pub free_slot_bits: u128,
    pub is_bid_bits: u128,
    pub orders: [u128; 128],
    // Using Option<NonZeroU64> in a pod type requires nightly
    pub client_order_ids: [u64; 128],
    pub referrer_rebates_accrued: u64,
}
unsafe impl Pod for OpenOrders {}
unsafe impl Zeroable for OpenOrders {}

impl OpenOrders {
    fn check_flags(&self) -> DexResult {
        let flags = BitFlags::from_bits(self.account_flags)
            .map_err(|_| DexErrorCode::InvalidMarketFlags)?;
        let required_flags = AccountFlag::Initialized | AccountFlag::OpenOrders;
        if flags != required_flags {
            Err(DexErrorCode::WrongOrdersAccount)?
        }
        Ok(())
    }

    fn init(&mut self, market: &[u64; 4], owner: &[u64; 4]) -> DexResult<()> {
        check_assert_eq!(self.account_flags, 0)?;
        self.account_flags = (AccountFlag::Initialized | AccountFlag::OpenOrders).bits();
        self.market = *market;
        self.owner = *owner;
        self.native_coin_total = 0;
        self.native_coin_free = 0;
        self.native_pc_total = 0;
        self.native_pc_free = 0;
        self.free_slot_bits = std::u128::MAX;
        Ok(())
    }

    #[inline]
    pub fn load_checked<'a>(
        orders_account: &'a AccountInfo,
        market_account: Option<&AccountInfo>,
        owner_account: Option<&AccountInfo>,
        program_id: &Pubkey,
    ) -> DexResult<Ref<'a, Self>> {
        check_assert_eq!(orders_account.owner, program_id)?;

        let account_data = orders_account.try_borrow_data()?;
        check_assert!(account_data.len() >= 12)?;
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        check_assert_eq!(head, ACCOUNT_HEAD_PADDING)?;
        check_assert_eq!(tail, ACCOUNT_TAIL_PADDING)?;

        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| {
            bytemuck::from_bytes(&account_data[5..account_data.len() - 7])
        });

        state.check_flags()?;
        if let Some(market_acc) = market_account {
            check_assert_eq!(identity(state.market), market_acc.key.to_aligned_bytes())
                .map_err(|_| DexErrorCode::WrongOrdersAccount)?;
        }

        if let Some(owner) = owner_account {
            check_assert_eq!(&identity(state.owner), &owner.key.to_aligned_bytes())
                .map_err(|_| DexErrorCode::WrongOrdersAccount)?;
        }
        Ok(state)
    }

    fn credit_locked_coin(&mut self, native_coin_amount: u64) {
        self.native_coin_total = self
            .native_coin_total
            .checked_add(native_coin_amount)
            .unwrap();
    }

    fn credit_locked_pc(&mut self, native_pc_amount: u64) {
        self.native_pc_total = self.native_pc_total.checked_add(native_pc_amount).unwrap();
    }

    fn lock_free_coin(&mut self, native_coin_amount: u64) {
        self.native_coin_free = self
            .native_coin_free
            .checked_sub(native_coin_amount)
            .unwrap();
    }

    fn lock_free_pc(&mut self, native_pc_amount: u64) {
        self.native_pc_free = self.native_pc_free.checked_sub(native_pc_amount).unwrap();
    }

    pub fn unlock_coin(&mut self, native_coin_amount: u64) {
        self.native_coin_free = self
            .native_coin_free
            .checked_add(native_coin_amount)
            .unwrap();
        assert!(self.native_coin_free <= self.native_coin_total);
    }

    pub fn unlock_pc(&mut self, native_pc_amount: u64) {
        self.native_pc_free = self.native_pc_free.checked_add(native_pc_amount).unwrap();
        assert!(self.native_pc_free <= self.native_pc_total);
    }

    fn slot_is_free(&self, slot: u8) -> bool {
        let slot_mask = 1u128 << slot;
        self.free_slot_bits & slot_mask != 0
    }

    #[inline]
    fn iter_filled_slots(&self) -> impl Iterator<Item = u8> {
        struct Iter {
            bits: u128,
        }
        impl Iterator for Iter {
            type Item = u8;
            #[inline(always)]
            fn next(&mut self) -> Option<Self::Item> {
                if self.bits == 0 {
                    None
                } else {
                    let next = self.bits.trailing_zeros();
                    let mask = 1u128 << next;
                    self.bits &= !mask;
                    Some(next as u8)
                }
            }
        }
        Iter {
            bits: !self.free_slot_bits,
        }
    }

    #[inline]
    fn orders_with_client_ids(&self) -> impl Iterator<Item = (NonZeroU64, u128, Side)> + '_ {
        self.iter_filled_slots().filter_map(move |slot| {
            let client_order_id = NonZeroU64::new(self.client_order_ids[slot as usize])?;
            let order_id = self.orders[slot as usize];
            let side = self.slot_side(slot).unwrap();
            Some((client_order_id, order_id, side))
        })
    }

    pub fn slot_side(&self, slot: u8) -> Option<Side> {
        let slot_mask = 1u128 << slot;
        if self.free_slot_bits & slot_mask != 0 {
            None
        } else if self.is_bid_bits & slot_mask != 0 {
            Some(Side::Bid)
        } else {
            Some(Side::Ask)
        }
    }

    pub fn remove_order(&mut self, slot: u8) -> DexResult {
        check_assert!(slot < 128)?;
        check_assert!(!self.slot_is_free(slot))?;

        let slot_mask = 1u128 << slot;
        self.orders[slot as usize] = 0;
        self.client_order_ids[slot as usize] = 0;
        self.free_slot_bits |= slot_mask;
        self.is_bid_bits &= !slot_mask;

        Ok(())
    }

    fn add_order(&mut self, id: u128, side: Side) -> DexResult<u8> {
        if self.free_slot_bits == 0 {
            Err(DexErrorCode::TooManyOpenOrders)?;
        }
        let slot = self.free_slot_bits.trailing_zeros();
        check_assert!(self.slot_is_free(slot as u8))?;
        let slot_mask = 1u128 << slot;
        self.free_slot_bits &= !slot_mask;
        match side {
            Side::Bid => {
                self.is_bid_bits |= slot_mask;
            }
            Side::Ask => {
                self.is_bid_bits &= !slot_mask;
            }
        };
        self.orders[slot as usize] = id;
        Ok(slot as u8)
    }
}

pub trait QueueHeader: Pod {
    type Item: Pod + Copy;

    fn head(&self) -> u64;
    fn set_head(&mut self, value: u64);
    fn count(&self) -> u64;
    fn set_count(&mut self, value: u64);

    fn incr_event_id(&mut self);
    fn decr_event_id(&mut self, n: u64);
}

pub struct Queue<'a, H: QueueHeader> {
    header: RefMut<'a, H>,
    buf: RefMut<'a, [H::Item]>,
}

impl<'a, H: QueueHeader> Queue<'a, H> {
    pub fn new(header: RefMut<'a, H>, buf: RefMut<'a, [H::Item]>) -> Self {
        Self { header, buf }
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.header.count()
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.header.count() as usize == self.buf.len()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.header.count() == 0
    }

    #[inline]
    pub fn push_back(&mut self, value: H::Item) -> Result<(), H::Item> {
        if self.full() {
            return Err(value);
        }
        let slot = ((self.header.head() + self.header.count()) as usize) % self.buf.len();
        self.buf[slot] = value;

        let count = self.header.count();
        self.header.set_count(count + 1);

        self.header.incr_event_id();
        Ok(())
    }

    #[inline]
    pub fn peek_front(&self) -> Option<&H::Item> {
        if self.empty() {
            return None;
        }
        Some(&self.buf[self.header.head() as usize])
    }

    #[inline]
    pub fn peek_front_mut(&mut self) -> Option<&mut H::Item> {
        if self.empty() {
            return None;
        }
        Some(&mut self.buf[self.header.head() as usize])
    }

    #[inline]
    pub fn pop_front(&mut self) -> Result<H::Item, ()> {
        if self.empty() {
            return Err(());
        }
        let value = self.buf[self.header.head() as usize];

        let count = self.header.count();
        self.header.set_count(count - 1);

        let head = self.header.head();
        self.header.set_head((head + 1) % self.buf.len() as u64);

        Ok(value)
    }

    #[inline]
    pub fn revert_pushes(&mut self, desired_len: u64) -> DexResult<()> {
        check_assert!(desired_len <= self.header.count())?;
        let len_diff = self.header.count() - desired_len;
        self.header.set_count(desired_len);
        self.header.decr_event_id(len_diff);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &H::Item> {
        QueueIterator {
            queue: self,
            index: 0,
        }
    }
}

struct QueueIterator<'a, 'b, H: QueueHeader> {
    queue: &'b Queue<'a, H>,
    index: u64,
}

impl<'a, 'b, H: QueueHeader> Iterator for QueueIterator<'a, 'b, H> {
    type Item = &'b H::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.queue.len() {
            None
        } else {
            let item = &self.queue.buf
                [(self.queue.header.head() + self.index) as usize % self.queue.buf.len()];
            self.index += 1;
            Some(item)
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct RequestQueueHeader {
    account_flags: u64, // Initialized, RequestQueue
    head: u64,
    count: u64,
    next_seq_num: u64,
}
unsafe impl Zeroable for RequestQueueHeader {}
unsafe impl Pod for RequestQueueHeader {}

impl QueueHeader for RequestQueueHeader {
    type Item = Request;

    fn head(&self) -> u64 {
        self.head
    }
    fn set_head(&mut self, value: u64) {
        self.head = value;
    }
    fn count(&self) -> u64 {
        self.count
    }
    fn set_count(&mut self, value: u64) {
        self.count = value;
    }
    #[inline(always)]
    fn incr_event_id(&mut self) {}
    #[inline(always)]
    fn decr_event_id(&mut self, _n: u64) {}
}

pub type RequestQueue<'a> = Queue<'a, RequestQueueHeader>;

impl RequestQueue<'_> {
    fn gen_order_id(&mut self, limit_price: u64, side: Side) -> u128 {
        let seq_num = self.gen_seq_num();
        let upper = (limit_price as u128) << 64;
        let lower = match side {
            Side::Bid => !seq_num,
            Side::Ask => seq_num,
        };
        upper | (lower as u128)
    }

    fn gen_seq_num(&mut self) -> u64 {
        let seq_num = self.header.next_seq_num;
        self.header.next_seq_num += 1;
        seq_num
    }
}

#[derive(Copy, Clone, BitFlags, Debug)]
#[repr(u8)]
enum RequestFlag {
    NewOrder = 0x01,
    CancelOrder = 0x02,
    Bid = 0x04,
    PostOnly = 0x08,
    ImmediateOrCancel = 0x10,
    DecrementTakeOnSelfTrade = 0x20,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Request {
    request_flags: u8,
    owner_slot: u8,
    fee_tier: u8,
    self_trade_behavior: u8,
    padding: [u8; 4],
    max_coin_qty_or_cancel_id: u64,
    native_pc_qty_locked: u64,
    order_id: u128,
    owner: [u64; 4],
    client_order_id: u64,
}
unsafe impl Zeroable for Request {}
unsafe impl Pod for Request {}

#[derive(Debug)]
pub enum RequestView {
    NewOrder {
        side: Side,
        order_type: OrderType,
        owner_slot: u8,
        fee_tier: FeeTier,
        order_id: u128,
        max_coin_qty: NonZeroU64,
        native_pc_qty_locked: Option<NonZeroU64>,
        owner: [u64; 4],
        client_order_id: Option<NonZeroU64>,
        self_trade_behavior: SelfTradeBehavior,
    },
    CancelOrder {
        side: Side,
        order_id: u128,
        cancel_id: u64,
        expected_owner_slot: u8,
        expected_owner: [u64; 4],
        client_order_id: Option<NonZeroU64>,
    },
}

impl Request {
    #[inline(always)]
    pub fn new(view: RequestView) -> Self {
        match view {
            RequestView::NewOrder {
                side,
                order_type,
                owner_slot,
                fee_tier,
                order_id,
                owner,
                max_coin_qty,
                native_pc_qty_locked,
                client_order_id,
                self_trade_behavior,
            } => {
                let mut flags = BitFlags::from_flag(RequestFlag::NewOrder);
                if side == Side::Bid {
                    flags.insert(RequestFlag::Bid);
                }
                match order_type {
                    OrderType::PostOnly => flags |= RequestFlag::PostOnly,
                    OrderType::ImmediateOrCancel => flags |= RequestFlag::ImmediateOrCancel,
                    OrderType::Limit => (),
                };

                Request {
                    request_flags: flags.bits(),
                    owner_slot,
                    fee_tier: fee_tier.into(),
                    self_trade_behavior: self_trade_behavior.into(),
                    padding: Zeroable::zeroed(),
                    order_id,
                    owner,
                    max_coin_qty_or_cancel_id: max_coin_qty.get(),
                    native_pc_qty_locked: native_pc_qty_locked.map_or(0, NonZeroU64::get),
                    client_order_id: client_order_id.map_or(0, NonZeroU64::get),
                }
            }
            RequestView::CancelOrder {
                side,
                expected_owner_slot,
                order_id,
                expected_owner,
                cancel_id,
                client_order_id,
            } => {
                let mut flags = BitFlags::from_flag(RequestFlag::CancelOrder);
                if side == Side::Bid {
                    flags.insert(RequestFlag::Bid);
                }
                Request {
                    request_flags: flags.bits(),
                    max_coin_qty_or_cancel_id: cancel_id,
                    order_id,
                    owner_slot: expected_owner_slot,
                    fee_tier: 0,
                    self_trade_behavior: 0,
                    owner: expected_owner,
                    native_pc_qty_locked: 0,
                    padding: Zeroable::zeroed(),
                    client_order_id: client_order_id.map_or(0, NonZeroU64::get),
                }
            }
        }
    }

    #[inline(always)]
    pub fn as_view(&self) -> DexResult<RequestView> {
        let flags = BitFlags::from_bits(self.request_flags).unwrap();
        let side = if flags.contains(RequestFlag::Bid) {
            Side::Bid
        } else {
            Side::Ask
        };
        if flags.contains(RequestFlag::NewOrder) {
            let allowed_flags = {
                use RequestFlag::*;
                NewOrder | Bid | PostOnly | ImmediateOrCancel
            };
            check_assert!(allowed_flags.contains(flags))?;
            let post_only = flags.contains(RequestFlag::PostOnly);
            let ioc = flags.contains(RequestFlag::ImmediateOrCancel);
            let order_type = match (post_only, ioc) {
                (true, false) => OrderType::PostOnly,
                (false, true) => OrderType::ImmediateOrCancel,
                (false, false) => OrderType::Limit,
                (true, true) => unreachable!(),
            };
            let fee_tier = FeeTier::try_from_primitive(self.fee_tier).or(check_unreachable!())?;
            let self_trade_behavior =
                SelfTradeBehavior::try_from_primitive(self.self_trade_behavior)
                    .or(check_unreachable!())?;
            Ok(RequestView::NewOrder {
                side,
                order_type,
                owner_slot: self.owner_slot,
                fee_tier,
                self_trade_behavior,
                order_id: self.order_id,
                owner: self.owner,
                max_coin_qty: NonZeroU64::new(self.max_coin_qty_or_cancel_id).unwrap(),
                native_pc_qty_locked: NonZeroU64::new(self.native_pc_qty_locked),
                client_order_id: NonZeroU64::new(self.client_order_id),
            })
        } else {
            check_assert!(flags.contains(RequestFlag::CancelOrder))?;
            let allowed_flags = {
                use RequestFlag::*;
                CancelOrder | Bid
            };
            check_assert!(allowed_flags.contains(flags))?;
            Ok(RequestView::CancelOrder {
                side,
                cancel_id: self.max_coin_qty_or_cancel_id,
                order_id: self.order_id,
                expected_owner_slot: self.owner_slot,
                expected_owner: self.owner,
                client_order_id: NonZeroU64::new(self.client_order_id),
            })
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct EventQueueHeader {
    account_flags: u64, // Initialized, EventQueue
    head: u64,
    count: u64,
    seq_num: u64,
}
unsafe impl Zeroable for EventQueueHeader {}
unsafe impl Pod for EventQueueHeader {}

unsafe impl TriviallyTransmutable for EventQueueHeader {}
unsafe impl TriviallyTransmutable for RequestQueueHeader {}

impl QueueHeader for EventQueueHeader {
    type Item = Event;

    fn head(&self) -> u64 {
        self.head
    }
    fn set_head(&mut self, value: u64) {
        self.head = value;
    }
    fn count(&self) -> u64 {
        self.count
    }
    fn set_count(&mut self, value: u64) {
        self.count = value;
    }
    fn incr_event_id(&mut self) {
        self.seq_num += 1;
    }
    fn decr_event_id(&mut self, n: u64) {
        self.seq_num -= n;
    }
}

pub type EventQueue<'a> = Queue<'a, EventQueueHeader>;

#[derive(Copy, Clone, BitFlags, Debug)]
#[repr(u8)]
enum EventFlag {
    Fill = 0x1,
    Out = 0x2,
    Bid = 0x4,
    Maker = 0x8,
    ReleaseFunds = 0x10,
}

impl EventFlag {
    #[inline]
    fn from_side(side: Side) -> BitFlags<Self> {
        match side {
            Side::Bid => EventFlag::Bid.into(),
            Side::Ask => BitFlags::empty(),
        }
    }

    #[inline]
    fn flags_to_side(flags: BitFlags<Self>) -> Side {
        if flags.contains(EventFlag::Bid) {
            Side::Bid
        } else {
            Side::Ask
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Event {
    event_flags: u8,
    owner_slot: u8,

    fee_tier: u8,

    _padding: [u8; 5],

    native_qty_released: u64,
    native_qty_paid: u64,
    native_fee_or_rebate: u64,

    order_id: u128,
    pub owner: [u64; 4],
    client_order_id: u64,
}
unsafe impl Zeroable for Event {}
unsafe impl Pod for Event {}

unsafe impl TriviallyTransmutable for Event {}
unsafe impl TriviallyTransmutable for Request {}

impl Event {
    #[inline(always)]
    pub fn new(view: EventView) -> Self {
        match view {
            EventView::Fill {
                side,
                maker,
                native_qty_paid,
                native_qty_received,
                native_fee_or_rebate,
                order_id,
                owner,
                owner_slot,
                fee_tier,
                client_order_id,
            } => {
                let maker_flag = if maker {
                    BitFlags::from_flag(EventFlag::Maker).bits()
                } else {
                    0
                };
                let event_flags =
                    (EventFlag::from_side(side) | EventFlag::Fill).bits() | maker_flag;
                Event {
                    event_flags,
                    owner_slot,
                    fee_tier: fee_tier.into(),

                    _padding: Zeroable::zeroed(),

                    native_qty_released: native_qty_received,
                    native_qty_paid,
                    native_fee_or_rebate,

                    order_id,
                    owner,

                    client_order_id: client_order_id.map_or(0, NonZeroU64::get),
                }
            }

            EventView::Out {
                side,
                release_funds,
                native_qty_unlocked,
                native_qty_still_locked,
                order_id,
                owner,
                owner_slot,
                client_order_id,
            } => {
                let release_funds_flag = if release_funds {
                    BitFlags::from_flag(EventFlag::ReleaseFunds).bits()
                } else {
                    0
                };
                let event_flags =
                    (EventFlag::from_side(side) | EventFlag::Out).bits() | release_funds_flag;
                Event {
                    event_flags,
                    owner_slot,
                    fee_tier: 0,

                    _padding: Zeroable::zeroed(),

                    native_qty_released: native_qty_unlocked,
                    native_qty_paid: native_qty_still_locked,
                    native_fee_or_rebate: 0,

                    order_id,
                    owner,
                    client_order_id: client_order_id.map_or(0, NonZeroU64::get),
                }
            }
        }
    }

    #[inline(always)]
    pub fn as_view(&self) -> DexResult<EventView> {
        let flags = BitFlags::from_bits(self.event_flags).unwrap();
        let side = EventFlag::flags_to_side(flags);
        let client_order_id = NonZeroU64::new(self.client_order_id);
        if flags.contains(EventFlag::Fill) {
            let allowed_flags = {
                use EventFlag::*;
                Fill | Bid | Maker
            };
            check_assert!(allowed_flags.contains(flags))?;

            return Ok(EventView::Fill {
                side,
                maker: flags.contains(EventFlag::Maker),
                native_qty_paid: self.native_qty_paid,
                native_qty_received: self.native_qty_released,
                native_fee_or_rebate: self.native_fee_or_rebate,

                order_id: self.order_id,
                owner: self.owner,

                owner_slot: self.owner_slot,
                fee_tier: self.fee_tier.try_into().or(check_unreachable!())?,
                client_order_id,
            });
        }
        let allowed_flags = {
            use EventFlag::*;
            Out | Bid | ReleaseFunds
        };
        check_assert!(allowed_flags.contains(flags))?;
        Ok(EventView::Out {
            side,
            release_funds: flags.contains(EventFlag::ReleaseFunds),
            native_qty_unlocked: self.native_qty_released,
            native_qty_still_locked: self.native_qty_paid,

            order_id: self.order_id,
            owner: self.owner,

            owner_slot: self.owner_slot,
            client_order_id,
        })
    }
}

#[derive(Debug)]
pub enum EventView {
    Fill {
        side: Side,
        maker: bool,
        native_qty_paid: u64,
        native_qty_received: u64,
        native_fee_or_rebate: u64,
        order_id: u128,
        owner: [u64; 4],
        owner_slot: u8,
        fee_tier: FeeTier,
        client_order_id: Option<NonZeroU64>,
    },
    Out {
        side: Side,
        release_funds: bool,
        native_qty_unlocked: u64,
        native_qty_still_locked: u64,
        order_id: u128,
        owner: [u64; 4],
        owner_slot: u8,
        client_order_id: Option<NonZeroU64>,
    },
}

impl EventView {
    fn side(&self) -> Side {
        match self {
            &EventView::Fill { side, .. } | &EventView::Out { side, .. } => side,
        }
    }
}

#[event]
pub struct FillEventLog {
    market: Pubkey,
    open_orders: Pubkey,
    open_orders_owner: Pubkey,
    bid: bool,
    maker: bool,
    native_qty_paid: u64,
    native_qty_received: u64,
    native_fee_or_rebate: u64,
    order_id: u128,
    owner_slot: u8,
    fee_tier: u8,
    client_order_id: Option<u64>,
    referrer_rebate: Option<u64>,
}

#[event]
pub struct OpenOrdersBalanceLog {
    open_orders: Pubkey,
    open_orders_owner: Pubkey,
    market: Pubkey,
    native_pc_total: u64,
    native_coin_total: u64,
    native_coin_free: u64,
    native_pc_free: u64,
    referrer_rebates_accrued: u64,
}

#[derive(Copy, Clone)]
#[repr(packed)]
struct OrderBookStateHeader {
    account_flags: u64, // Initialized, (Bids or Asks)
}
unsafe impl Zeroable for OrderBookStateHeader {}
unsafe impl Pod for OrderBookStateHeader {}

pub enum State {}

fn gen_vault_signer_seeds<'a>(nonce: &'a u64, market: &'a Pubkey) -> [&'a [u8]; 2] {
    [market.as_ref(), bytes_of(nonce)]
}

#[cfg(not(any(test, feature = "fuzz")))]
#[inline]
pub fn gen_vault_signer_key(
    nonce: u64,
    market: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    let seeds = gen_vault_signer_seeds(&nonce, market);
    Ok(Pubkey::create_program_address(&seeds, program_id)?)
}

#[cfg(any(test, feature = "fuzz"))]
pub fn gen_vault_signer_key(
    nonce: u64,
    market: &Pubkey,
    _program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    gen_vault_signer_seeds(&nonce, market);
    Ok(Pubkey::default())
}

#[cfg(not(any(test, feature = "fuzz")))]
fn invoke_spl_token(
    instruction: &solana_program::instruction::Instruction,
    account_infos: &[AccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> solana_program::entrypoint::ProgramResult {
    solana_program::program::invoke_signed(instruction, account_infos, signers_seeds)
}

#[cfg(any(test, feature = "fuzz"))]
fn invoke_spl_token(
    instruction: &solana_program::instruction::Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> solana_program::entrypoint::ProgramResult {
    assert_eq!(instruction.program_id, spl_token::ID);
    let account_infos: Vec<AccountInfo> = instruction
        .accounts
        .iter()
        .map(|meta| {
            account_infos
                .iter()
                .find(|info| *info.key == meta.pubkey)
                .unwrap()
                .clone()
        })
        .collect();
    spl_token::processor::Processor::process(
        &instruction.program_id,
        &account_infos,
        &instruction.data,
    )?;
    Ok(())
}

#[cfg(feature = "program")]
fn send_from_vault<'a, 'b: 'a>(
    native_amount: u64,
    recipient: account_parser::TokenAccount<'a, 'b>,
    vault: account_parser::TokenAccount<'a, 'b>,
    spl_token_program: account_parser::SplTokenProgram<'a, 'b>,
    vault_signer: account_parser::VaultSigner<'a, 'b>,
    vault_signer_seeds: &[&[u8]],
) -> DexResult {
    let deposit_instruction = spl_token::instruction::transfer(
        &spl_token::ID,
        vault.inner().key,
        recipient.inner().key,
        &vault_signer.inner().key,
        &[],
        native_amount,
    )?;
    let accounts: &[AccountInfo] = &[
        vault.inner().clone(),
        recipient.inner().clone(),
        vault_signer.inner().clone(),
        spl_token_program.inner().clone(),
    ];
    invoke_spl_token(&deposit_instruction, &accounts[..], &[vault_signer_seeds])
        .map_err(|_| DexErrorCode::TransferFailed)?;
    Ok(())
}

#[cfg(feature = "fuzz")]
pub mod fuzz_account_parser {
    pub use super::account_parser::SignerAccount;
}

pub(crate) mod account_parser {
    use super::*;

    macro_rules! declare_validated_account_wrapper {
        ($WrapperT:ident, $validate:expr $(, $a:ident : $t:ty)*) => {
            #[derive(Copy, Clone)]
            pub struct $WrapperT<'a, 'b: 'a>(&'a AccountInfo<'b>);
            impl<'a, 'b: 'a> $WrapperT<'a, 'b> {
                pub fn new(account: &'a AccountInfo<'b> $(,$a: $t)*) -> DexResult<Self> {
                    let validate_result: DexResult = $validate(account $(,$a)*);
                    validate_result?;
                    Ok($WrapperT(account))
                }

                #[inline(always)]
                #[allow(unused)]
                pub fn inner(self) -> &'a AccountInfo<'b> {
                    self.0
                }
            }
        }
    }

    declare_validated_account_wrapper!(SplTokenProgram, |account: &AccountInfo| {
        check_assert_eq!(*account.key, spl_token::ID)?;
        Ok(())
    });

    declare_validated_account_wrapper!(TokenMint, |mint: &AccountInfo| {
        check_assert_eq!(*mint.owner, spl_token::ID)?;
        let data = mint.try_borrow_data()?;
        check_assert_eq!(data.len(), spl_token::state::Mint::LEN)?;

        let is_initialized = data[0x2d];
        check_assert_eq!(is_initialized, 1u8)?;
        Ok(())
    });

    declare_validated_account_wrapper!(TokenAccount, |account: &AccountInfo| {
        check_assert_eq!(*account.owner, spl_token::ID)?;
        let data = account.try_borrow_data()?;
        check_assert_eq!(data.len(), spl_token::state::Account::LEN)?;

        let is_initialized = data[0x6c];
        check_assert_eq!(is_initialized, 1u8)?;
        Ok(())
    });

    macro_rules! declare_validated_token_account_wrapper {
        ($WrapperT:ident, $validate:expr $(, $a:ident : $t:ty)*) => {
            #[derive(Copy, Clone)]
            pub struct $WrapperT<'a, 'b: 'a>(TokenAccount<'a, 'b>);
            impl<'a, 'b: 'a> $WrapperT<'a, 'b> {
                fn new(token_account: TokenAccount<'a, 'b> $(,$a: $t)*) -> DexResult<Self> {
                    let validate_result: DexResult = $validate(token_account $(,$a)*);
                    validate_result?;
                    Ok($WrapperT(token_account))
                }

                fn from_account(account: &'a AccountInfo<'b> $(,$a: $t)*) -> DexResult<Self> {
                    let token_account = TokenAccount::new(account)?;
                    Self::new(token_account $(,$a)*)
                }

                #[inline(always)]
                pub fn token_account(self) -> TokenAccount<'a, 'b> {
                    self.0
                }

                #[inline(always)]
                #[allow(unused)]
                pub fn account(self) -> &'a AccountInfo<'b> {
                    self.0.inner()
                }
            }
        }
    }

    declare_validated_account_wrapper!(SignerAccount, |account: &AccountInfo| {
        check_assert!(account.is_signer)?;
        Ok(())
    });

    declare_validated_account_wrapper!(SigningFeeSweeper, |account: &AccountInfo| {
        check_assert!(account.is_signer)?;
        check_assert_eq!(account.key, &fee_sweeper::ID)?;
        Ok(())
    });

    declare_validated_account_wrapper!(SigningDisableAuthority, |account: &AccountInfo| {
        check_assert!(account.is_signer)?;
        check_assert_eq!(account.key, &disable_authority::ID)?;
        Ok(())
    });

    declare_validated_token_account_wrapper!(
        CoinVault,
        |token_account: TokenAccount, market: &Market| { market.check_coin_vault(token_account) },
        market: &Market
    );

    declare_validated_token_account_wrapper!(
        PcVault,
        |token_account: TokenAccount, market: &Market| { market.check_pc_vault(token_account) },
        market: &Market
    );

    declare_validated_token_account_wrapper!(
        CoinWallet,
        |token_account: TokenAccount, market: &Market| { market.check_coin_payer(token_account) },
        market: &Market
    );

    declare_validated_token_account_wrapper!(
        PcWallet,
        |token_account: TokenAccount, market: &Market| { market.check_pc_payer(token_account) },
        market: &Market
    );

    declare_validated_account_wrapper!(
        VaultSigner,
        |account: &AccountInfo, market: &Market, program_id: &Pubkey| {
            let vault_signer_key =
                gen_vault_signer_key(market.vault_signer_nonce, &market.pubkey(), program_id)?;
            Ok(check_assert_eq!(&vault_signer_key, account.key)?)
        },
        market: &Market,
        program_id: &Pubkey
    );

    impl<'a, 'b: 'a> TokenAccount<'a, 'b> {
        pub fn balance(self) -> DexResult<u64> {
            let data = self.inner().try_borrow_data()?;
            Ok(u64::from_le_bytes(*array_ref![data, 64, 8]))
        }
    }

    #[derive(Copy, Clone)]
    pub struct TokenAccountAndMint<'a, 'b: 'a> {
        account: TokenAccount<'a, 'b>,
        mint: TokenMint<'a, 'b>,
    }

    impl<'a, 'b: 'a> TokenAccountAndMint<'a, 'b> {
        fn new(account: TokenAccount<'a, 'b>, mint: TokenMint<'a, 'b>) -> DexResult<Self> {
            let account_data = account.0.try_borrow_data()?;
            check_assert_eq!(mint.0.key.as_ref(), &account_data[..32])?;
            Ok(TokenAccountAndMint { account, mint })
        }

        pub fn get_account(self) -> TokenAccount<'a, 'b> {
            self.account
        }

        pub fn get_mint(self) -> TokenMint<'a, 'b> {
            self.mint
        }
    }

    pub struct InitializeMarketArgs<'a, 'b: 'a> {
        pub program_id: &'a Pubkey,
        pub instruction: &'a InitializeMarketInstruction,
        serum_dex_accounts: &'a [AccountInfo<'b>; 5],
        pub coin_vault_and_mint: TokenAccountAndMint<'a, 'b>,
        pub pc_vault_and_mint: TokenAccountAndMint<'a, 'b>,
        pub market_authority: Option<&'a AccountInfo<'b>>,
        pub prune_authority: Option<&'a AccountInfo<'b>>,
        pub consume_events_authority: Option<&'a AccountInfo<'b>>,
    }

    impl<'a, 'b: 'a> InitializeMarketArgs<'a, 'b> {
        pub fn new(
            program_id: &'a Pubkey,
            instruction: &'a InitializeMarketInstruction,
            accounts: &'a [AccountInfo<'b>],
        ) -> DexResult<Self> {
            check_assert!(accounts.len() >= 10 && accounts.len() <= 13)?;
            let (
                unchecked_serum_dex_accounts,
                unchecked_vaults,
                unchecked_mints,
                unchecked_rent,
                remaining_accounts,
            ) = array_refs![accounts, 5, 2, 2, 1; .. ;];

            let mut rem_iter = remaining_accounts.iter();
            let market_authority = rem_iter.next();
            let prune_authority = rem_iter.next();
            let consume_events_authority = rem_iter.next();

            {
                // Dynamic sysvars don't work in unit tests.
                #[cfg(any(test, feature = "fuzz"))]
                let rent = Rent::from_account_info(&unchecked_rent[0])?;
                #[cfg(not(any(test, feature = "fuzz")))]
                let rent = Rent::get()?;

                let end_idx = accounts.len() - remaining_accounts.len() - 1;
                for account in &accounts[1..end_idx] {
                    let data_len = account.data_len();
                    let lamports = account.lamports();
                    check_assert!(rent.is_exempt(lamports, data_len))?;
                }
            }

            let mut checked_vaults = [None, None];
            for account in unchecked_serum_dex_accounts {
                check_assert_eq!(account.owner, program_id)?;
                let data = account.try_borrow_data()?;
                check_assert_eq!(data.len() % 8, 4)?;
                check_assert!(data.len() >= 20)?;
                let (padding5, header, _, padding7) = array_refs![&data, 5, 8; .. ; 7];
                check_assert_eq!(*padding5, [0u8; 5])?;
                check_assert_eq!(*header, [0u8; 8])?;
                check_assert_eq!(*padding7, [0u8; 7])?;
            }
            let serum_dex_accounts = unchecked_serum_dex_accounts;
            let vault_owner_key_bytes = gen_vault_signer_key(
                instruction.vault_signer_nonce,
                serum_dex_accounts[0].key,
                program_id,
            )?
            .to_bytes();
            for i in 0..=1 {
                let vault = TokenAccount::new(&unchecked_vaults[i])?;
                let mint = TokenMint::new(&unchecked_mints[i])?;

                // check that the vaults are owned by the market's withdrawal authority key
                let vault_data = vault.0.try_borrow_data()?;
                let vault_owner = array_ref![vault_data, 0x20, 0x20];
                check_assert_eq!(vault_owner, &vault_owner_key_bytes)?;

                // check that the vault has no delegate
                let delegate_tag = array_ref![vault_data, 0x48, 0x4];
                check_assert_eq!(*delegate_tag, [0u8; 4])?;

                // check that the vault has no close authority
                let close_authority_tag = array_ref![vault_data, 0x81, 0x4];
                check_assert_eq!(*close_authority_tag, [0u8; 4])?;

                checked_vaults[i] = Some(TokenAccountAndMint::new(vault, mint)?);
            }
            let [coin_vault_and_mint, pc_vault_and_mint] = match checked_vaults {
                [Some(cvm), Some(pvm)] => [cvm, pvm],
                _ => check_unreachable!()?,
            };

            Ok(InitializeMarketArgs {
                program_id,
                instruction,
                serum_dex_accounts,
                coin_vault_and_mint,
                pc_vault_and_mint,
                market_authority,
                prune_authority,
                consume_events_authority,
            })
        }

        pub fn get_market(&self) -> &'a AccountInfo<'b> {
            &self.serum_dex_accounts[0]
        }

        pub fn get_req_q(&self) -> &'a AccountInfo<'b> {
            &self.serum_dex_accounts[1]
        }

        pub fn get_event_q(&self) -> &'a AccountInfo<'b> {
            &self.serum_dex_accounts[2]
        }

        pub fn get_bids(&self) -> &'a AccountInfo<'b> {
            &self.serum_dex_accounts[3]
        }

        pub fn get_asks(&self) -> &'a AccountInfo<'b> {
            &self.serum_dex_accounts[4]
        }
    }

    pub struct SendTakeArgs<'a, 'b: 'a> {
        pub instruction: &'a SendTakeInstruction,
        pub signer: SignerAccount<'a, 'b>,
        pub req_q: RequestQueue<'a>,
        pub event_q: EventQueue<'a>,
        pub order_book_state: OrderBookState<'a>,
        pub coin_wallet: CoinWallet<'a, 'b>,
        pub pc_wallet: PcWallet<'a, 'b>,
        pub coin_vault: CoinVault<'a, 'b>,
        pub pc_vault: PcVault<'a, 'b>,
        pub spl_token_program: SplTokenProgram<'a, 'b>,
        pub fee_tier: FeeTier,
    }
    impl<'a, 'b: 'a> SendTakeArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            instruction: &'a SendTakeInstruction,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(SendTakeArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            const MIN_ACCOUNTS: usize = 11;
            check_assert!(accounts.len() == MIN_ACCOUNTS || accounts.len() == MIN_ACCOUNTS + 1)?;
            let (fixed_accounts, fee_discount_account): (
                &'a [AccountInfo<'b>; MIN_ACCOUNTS],
                &'a [AccountInfo<'b>],
            ) = array_refs![accounts, MIN_ACCOUNTS; .. ;];
            let &[
                ref market_acc,
                ref req_q_acc,
                ref event_q_acc,
                ref bids_acc,
                ref asks_acc,
                ref coin_wallet_acc,
                ref pc_wallet_acc,
                ref signer_acc,
                ref coin_vault_acc,
                ref pc_vault_acc,
                ref spl_token_program_acc,
            ]: &'a [AccountInfo<'b>; MIN_ACCOUNTS] = fixed_accounts;
            let srm_or_msrm_account = match fee_discount_account {
                &[] => None,
                &[ref account] => Some(TokenAccount::new(account)?),
                _ => check_unreachable!()?,
            };

            let mut market = Market::load(market_acc, program_id, false)?;

            let signer = SignerAccount::new(signer_acc)?;
            let fee_tier = market
                .load_fee_tier(&signer.inner().key.to_aligned_bytes(), srm_or_msrm_account)
                .or(check_unreachable!())?;
            let req_q = market.load_request_queue_mut(req_q_acc)?;
            let event_q = market.load_event_queue_mut(event_q_acc)?;

            let coin_wallet = CoinWallet::from_account(coin_wallet_acc, &market)?;
            let pc_wallet = PcWallet::from_account(pc_wallet_acc, &market)?;

            let coin_vault = CoinVault::from_account(coin_vault_acc, &market)?;
            let pc_vault = PcVault::from_account(pc_vault_acc, &market)?;

            let spl_token_program = SplTokenProgram::new(spl_token_program_acc)?;

            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;

            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = SendTakeArgs {
                instruction,
                signer,
                req_q,
                event_q,
                fee_tier,
                coin_wallet,
                pc_wallet,
                coin_vault,
                pc_vault,
                order_book_state,
                spl_token_program,
            };
            f(args)
        }
    }

    pub struct NewOrderV3Args<'a, 'b: 'a> {
        pub instruction: &'a NewOrderInstructionV3,
        pub open_orders: RefMut<'a, OpenOrders>,
        pub open_orders_address: [u64; 4],
        pub owner: SignerAccount<'a, 'b>,
        pub req_q: RequestQueue<'a>,
        pub event_q: EventQueue<'a>,
        pub order_book_state: OrderBookState<'a>,
        pub payer: TokenAccount<'a, 'b>,
        pub coin_vault: CoinVault<'a, 'b>,
        pub pc_vault: PcVault<'a, 'b>,
        pub spl_token_program: SplTokenProgram<'a, 'b>,
        pub fee_tier: FeeTier,
    }
    impl<'a, 'b: 'a> NewOrderV3Args<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            instruction: &'a NewOrderInstructionV3,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(NewOrderV3Args) -> DexResult<T>,
        ) -> DexResult<T> {
            const MIN_ACCOUNTS: usize = 12;
            check_assert!(
                accounts.len() == MIN_ACCOUNTS
                    || accounts.len() == MIN_ACCOUNTS + 1
                    || accounts.len() == MIN_ACCOUNTS + 2
            )?;
            let (fixed_accounts, fee_discount_account): (
                &'a [AccountInfo<'b>; MIN_ACCOUNTS],
                &'a [AccountInfo<'b>],
            ) = array_refs![accounts, MIN_ACCOUNTS; .. ;];
            let &[
                ref market_acc,
                ref open_orders_acc,
                ref req_q_acc,
                ref event_q_acc,
                ref bids_acc,
                ref asks_acc,
                ref payer_acc,
                ref owner_acc,
                ref coin_vault_acc,
                ref pc_vault_acc,
                ref spl_token_program_acc,
                ref rent_sysvar_acc,
            ]: &'a [AccountInfo<'b>; MIN_ACCOUNTS] = fixed_accounts;
            let srm_or_msrm_account = match fee_discount_account {
                &[] => None,
                &[ref account] => Some(TokenAccount::new(account)?),
                _ => check_unreachable!()?,
            };

            let mut market = Market::load(market_acc, program_id, false)?;

            // Dynamic sysvars don't work in unit tests.
            #[cfg(any(test, feature = "fuzz"))]
            let rent = Rent::from_account_info(rent_sysvar_acc)?;
            #[cfg(not(any(test, feature = "fuzz")))]
            let rent = Rent::get()?;

            let owner = SignerAccount::new(owner_acc)?;
            let fee_tier =
                market.load_fee_tier(&owner.inner().key.to_aligned_bytes(), srm_or_msrm_account)?;
            let open_orders_address = open_orders_acc.key.to_aligned_bytes();
            let req_q = market.load_request_queue_mut(req_q_acc)?;
            let event_q = market.load_event_queue_mut(event_q_acc)?;

            let payer = TokenAccount::new(payer_acc)?;
            match instruction.side {
                Side::Bid => market.check_pc_payer(payer).or(check_unreachable!())?,
                Side::Ask => market.check_coin_payer(payer).or(check_unreachable!())?,
            };
            let coin_vault = CoinVault::from_account(coin_vault_acc, &market)?;
            let pc_vault = PcVault::from_account(pc_vault_acc, &market)?;
            market.check_enabled()?;
            let spl_token_program = SplTokenProgram::new(spl_token_program_acc)?;

            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;

            let open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(owner.inner()),
                program_id,
                Some(rent),
                None, // To use an open orders authority, explicitly use the
                      // InitOpenOrders instruction.
            )?;
            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = NewOrderV3Args {
                instruction,
                order_book_state,
                open_orders,
                open_orders_address,
                owner,
                req_q,
                event_q,
                payer,
                coin_vault,
                pc_vault,
                spl_token_program,
                fee_tier,
            };
            f(args)
        }
    }

    pub struct ReplaceOrdersByClientIdsArgs<'a, 'b: 'a> {
        pub program_id: &'a Pubkey,
        pub cancel_accounts: &'a [AccountInfo<'b>],
        pub client_order_ids: [u64; 8],
        pub instructions: &'a Vec<NewOrderInstructionV3>,
        pub accounts: &'a [AccountInfo<'b>],
    }
    impl<'a, 'b: 'a> ReplaceOrdersByClientIdsArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            instructions: &'a Vec<NewOrderInstructionV3>,
            f: impl FnOnce(ReplaceOrdersByClientIdsArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            // Account indices for market, bids, asks, OpenOrders, owner, event_q
            let cancel_accounts = [0, 4, 5, 1, 7, 3]
                .iter()
                .map(|i| accounts[*i].clone())
                .collect::<Vec<_>>();
            let mut client_order_ids = [0; 8];
            for (instruction, client_order_id) in
                instructions.iter().zip(client_order_ids.iter_mut())
            {
                *client_order_id = instruction.client_order_id;
            }

            f(ReplaceOrdersByClientIdsArgs {
                program_id,
                instructions,
                accounts,
                cancel_accounts: cancel_accounts.as_slice(),
                client_order_ids,
            })
        }
    }

    pub struct ConsumeEventsArgs<'a, 'b: 'a> {
        pub limit: u16,
        pub program_id: &'a Pubkey,
        pub open_orders_accounts: &'a [AccountInfo<'b>],
        pub market: Market<'a>,
        pub event_q: EventQueue<'a>,
    }
    impl<'a, 'b: 'a> ConsumeEventsArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            limit: u16,
            f: impl FnOnce(ConsumeEventsArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() >= 5)?;
            #[rustfmt::skip]
            let (
                &[],
                open_orders_accounts,
                &[
                    ref market_acc,
                    ref event_q_acc,
                ],
                _
            ) = array_refs![accounts, 0; .. ; 2, 2];
            let market = Market::load(market_acc, program_id, true)?;
            check_assert!(market.consume_events_authority().is_none())?;
            let event_q = market.load_event_queue_mut(event_q_acc)?;
            let args = ConsumeEventsArgs {
                limit,
                program_id,
                open_orders_accounts,
                market,
                event_q,
            };
            f(args)
        }

        pub fn with_parsed_args_permissioned<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            limit: u16,
            f: impl FnOnce(ConsumeEventsArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() >= 4)?;
            #[rustfmt::skip]
            let (
                &[],
                open_orders_accounts,
                &[
                    ref market_acc,
                    ref event_q_acc,
                    ref consume_events_auth,
                ]
            ) = array_refs![accounts, 0; .. ; 3];
            let market = Market::load(market_acc, program_id, true)?;
            check_assert!(consume_events_auth.is_signer)?;
            check_assert_eq!(
                Some(consume_events_auth.key),
                market.consume_events_authority()
            )?;
            let event_q = market.load_event_queue_mut(event_q_acc)?;
            let args = ConsumeEventsArgs {
                limit,
                program_id,
                open_orders_accounts,
                market,
                event_q,
            };
            f(args)
        }
    }

    pub struct CancelOrderV2Args<'a, 'b: 'a> {
        pub instruction: &'a CancelOrderInstructionV2,
        pub open_orders_address: [u64; 4],
        pub open_orders: &'a mut OpenOrders,
        pub open_orders_signer: SignerAccount<'a, 'b>,
        pub order_book_state: OrderBookState<'a>,
        pub event_q: EventQueue<'a>,
    }
    impl<'a, 'b: 'a> CancelOrderV2Args<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            instruction: &'a CancelOrderInstructionV2,
            f: impl FnOnce(CancelOrderV2Args) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() >= 6)?;
            #[rustfmt::skip]
            let &[
                ref market_acc,
                ref bids_acc,
                ref asks_acc,
                ref open_orders_acc,
                ref open_orders_signer_acc,
                ref event_q_acc,
            ] = array_ref![accounts, 0, 6];

            let mut market = Market::load(market_acc, program_id, true).or(check_unreachable!())?;

            let open_orders_signer = SignerAccount::new(open_orders_signer_acc)?;
            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(open_orders_signer.inner()),
                program_id,
                None,
                None,
            )?;
            let open_orders_address = open_orders_acc.key.to_aligned_bytes();

            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;

            let event_q = market.load_event_queue_mut(event_q_acc)?;

            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = CancelOrderV2Args {
                instruction,
                open_orders_address,
                open_orders: open_orders.deref_mut(),
                open_orders_signer,
                order_book_state,
                event_q,
            };
            f(args)
        }
    }

    pub struct CancelOrderByClientIdV2Args<'a, 'b: 'a> {
        pub client_order_id: NonZeroU64,
        pub open_orders_address: [u64; 4],
        pub open_orders: &'a mut OpenOrders,
        pub open_orders_signer: SignerAccount<'a, 'b>,
        pub order_book_state: OrderBookState<'a>,
        pub event_q: EventQueue<'a>,
    }
    impl<'a, 'b: 'a> CancelOrderByClientIdV2Args<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            client_order_id: u64,
            f: impl FnOnce(CancelOrderByClientIdV2Args) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() >= 6)?;
            #[rustfmt::skip]
            let &[
                ref market_acc,
                ref bids_acc,
                ref asks_acc,
                ref open_orders_acc,
                ref open_orders_signer_acc,
                ref event_q_acc,
            ] = array_ref![accounts, 0, 6];

            let client_order_id = NonZeroU64::new(client_order_id).ok_or(assertion_error!())?;

            let mut market = Market::load(market_acc, program_id, true).or(check_unreachable!())?;

            let open_orders_signer = SignerAccount::new(open_orders_signer_acc)?;
            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(open_orders_signer.inner()),
                program_id,
                None,
                None,
            )?;
            let open_orders_address = open_orders_acc.key.to_aligned_bytes();

            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;

            let event_q = market.load_event_queue_mut(event_q_acc)?;

            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = CancelOrderByClientIdV2Args {
                client_order_id,
                open_orders_address,
                open_orders: open_orders.deref_mut(),
                open_orders_signer,
                order_book_state,
                event_q,
            };
            f(args)
        }
    }

    pub struct CancelOrdersByClientIdsArgs<'a, 'b: 'a> {
        pub client_order_ids: &'a [NonZeroU64],
        pub open_orders_address: [u64; 4],
        pub open_orders: &'a mut OpenOrders,
        pub open_orders_signer: SignerAccount<'a, 'b>,
        pub order_book_state: OrderBookState<'a>,
        pub event_q: EventQueue<'a>,
    }
    impl<'a, 'b: 'a> CancelOrdersByClientIdsArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            client_order_ids: [u64; 8],
            f: impl FnOnce(CancelOrdersByClientIdsArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() >= 6)?;
            #[rustfmt::skip]
            let &[
                ref market_acc,
                ref bids_acc,
                ref asks_acc,
                ref open_orders_acc,
                ref open_orders_signer_acc,
                ref event_q_acc,
            ] = array_ref![accounts, 0, 6];

            let client_order_ids = client_order_ids
                .iter()
                .cloned()
                .filter_map(NonZeroU64::new)
                .collect::<Vec<NonZeroU64>>();

            let mut market = Market::load(market_acc, program_id, true).or(check_unreachable!())?;

            let open_orders_signer = SignerAccount::new(open_orders_signer_acc)?;
            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(open_orders_signer.inner()),
                program_id,
                None,
                None,
            )?;
            let open_orders_address = open_orders_acc.key.to_aligned_bytes();

            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;

            let event_q = market.load_event_queue_mut(event_q_acc)?;

            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = CancelOrdersByClientIdsArgs {
                client_order_ids: client_order_ids.as_slice(),
                open_orders_address,
                open_orders: open_orders.deref_mut(),
                open_orders_signer,
                order_book_state,
                event_q,
            };
            f(args)
        }
    }

    pub struct SettleFundsArgs<'a, 'b: 'a> {
        pub market: Market<'a>,
        pub open_orders: &'a mut OpenOrders,
        pub coin_vault: CoinVault<'a, 'b>,
        pub pc_vault: PcVault<'a, 'b>,
        pub coin_wallet: CoinWallet<'a, 'b>,
        pub pc_wallet: PcWallet<'a, 'b>,
        pub vault_signer: VaultSigner<'a, 'b>,
        pub spl_token_program: SplTokenProgram<'a, 'b>,
        pub referrer: Option<PcWallet<'a, 'b>>,
    }
    impl<'a, 'b: 'a> SettleFundsArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(SettleFundsArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert!(accounts.len() == 9 || accounts.len() == 10)?;
            #[rustfmt::skip]
            let (&[
                ref market_acc,
                ref open_orders_acc,
                ref owner_acc,
                ref coin_vault_acc,
                ref pc_vault_acc,
                ref coin_wallet_acc,
                ref pc_wallet_acc,
                ref vault_signer_acc,
                ref spl_token_program_acc,
            ], remaining_accounts) = array_refs![accounts, 9; ..;];
            let spl_token_program = SplTokenProgram::new(spl_token_program_acc)?;
            let market = Market::load(market_acc, program_id, true)?;
            let owner = SignerAccount::new(owner_acc).or(check_unreachable!())?;

            let coin_vault =
                CoinVault::from_account(coin_vault_acc, &market).or(check_unreachable!())?;
            let pc_vault = PcVault::from_account(pc_vault_acc, &market).or(check_unreachable!())?;
            let coin_wallet =
                CoinWallet::from_account(coin_wallet_acc, &market).or(check_unreachable!())?;
            let pc_wallet =
                PcWallet::from_account(pc_wallet_acc, &market).or(check_unreachable!())?;

            let referrer = match remaining_accounts {
                &[] => None,
                &[ref referrer_acc] => {
                    Some(PcWallet::from_account(referrer_acc, &market).or(check_unreachable!())?)
                }
                _ => check_unreachable!()?,
            };

            let vault_signer = VaultSigner::new(vault_signer_acc, &market, program_id)?;

            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(owner.inner()),
                program_id,
                None,
                None,
            )?;

            let args = SettleFundsArgs {
                market,
                open_orders: open_orders.deref_mut(),
                coin_vault,
                pc_vault,
                coin_wallet,
                pc_wallet,
                vault_signer,
                spl_token_program,
                referrer,
            };
            f(args)
        }
    }

    pub struct DisableMarketArgs<'a, 'b: 'a> {
        pub market: &'a mut MarketState,
        pub authorization: SigningDisableAuthority<'a, 'b>,
    }
    impl<'a, 'b: 'a> DisableMarketArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(DisableMarketArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert_eq!(accounts.len(), 2)?;
            let &[ref market_acc, ref signer_acc] = array_ref![accounts, 0, 2];
            let mut market = Market::load(market_acc, program_id, false)?;
            let authorization = SigningDisableAuthority::new(signer_acc)?;

            let args = DisableMarketArgs {
                market: market.deref_mut(),
                authorization,
            };
            f(args)
        }
    }

    pub struct SweepFeesArgs<'a, 'b: 'a> {
        pub market: Market<'a>,
        pub pc_vault: PcVault<'a, 'b>,
        pub fee_receiver: PcWallet<'a, 'b>,
        pub vault_signer: VaultSigner<'a, 'b>,
        pub spl_token_program: SplTokenProgram<'a, 'b>,
        pub authorization: SigningFeeSweeper<'a, 'b>,
    }
    impl<'a, 'b: 'a> SweepFeesArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(SweepFeesArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            check_assert_eq!(accounts.len(), 6)?;
            #[rustfmt::skip]
            let &[
                ref market_acc,
                ref pc_vault_acc,
                ref sweep_authority_acc,
                ref pc_wallet_acc,
                ref vault_signer_acc,
                ref spl_token_program
            ] = array_ref![accounts, 0, 6];

            let market = Market::load(market_acc, program_id, false)?;
            let pc_vault = PcVault::from_account(pc_vault_acc, &market)?;
            let fee_receiver = PcWallet::from_account(pc_wallet_acc, &market)?;
            let vault_signer = VaultSigner::new(vault_signer_acc, &market, program_id)?;
            let spl_token_program = SplTokenProgram::new(spl_token_program)?;
            let authorization = SigningFeeSweeper::new(sweep_authority_acc)?;

            let args = SweepFeesArgs {
                market,
                pc_vault,
                fee_receiver,
                vault_signer,
                spl_token_program,
                authorization,
            };
            f(args)
        }
    }

    pub struct CloseOpenOrdersArgs<'a, 'b: 'a> {
        pub open_orders: &'a mut OpenOrders,
        pub open_orders_acc: &'a AccountInfo<'b>,
        pub dest_acc: &'a AccountInfo<'b>,
    }

    impl<'a, 'b: 'a> CloseOpenOrdersArgs<'a, 'b> {
        pub fn with_parsed_args<T>(
            program_id: &'a Pubkey,
            accounts: &'a [AccountInfo<'b>],
            f: impl FnOnce(CloseOpenOrdersArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            // Parse accounts.
            check_assert_eq!(accounts.len(), 4)?;
            #[rustfmt::skip]
            let &[
                ref open_orders_acc,
                ref owner_acc,
                ref dest_acc,
                ref market_acc,
            ] = array_ref![accounts, 0, 4];

            // Validate the accounts given are valid.
            let owner = SignerAccount::new(owner_acc)?;
            let market = Market::load(market_acc, program_id, true)?;
            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(owner.inner()),
                program_id,
                None,
                None,
            )?;

            // Only accounts with no funds associated with them can be closed.
            if open_orders.free_slot_bits != std::u128::MAX {
                return Err(DexErrorCode::TooManyOpenOrders.into());
            }
            if open_orders.native_coin_total != 0 {
                solana_program::msg!(
                    "Base currency total must be zero to close the open orders account"
                );
                return Err(DexErrorCode::TooManyOpenOrders.into());
            }
            if open_orders.native_pc_total != 0 {
                solana_program::msg!(
                    "Quote currency total must be zero to close the open orders account"
                );
                return Err(DexErrorCode::TooManyOpenOrders.into());
            }

            // Invoke processor.
            f(CloseOpenOrdersArgs {
                open_orders: open_orders.deref_mut(),
                open_orders_acc,
                dest_acc,
            })
        }
    }

    pub struct InitOpenOrdersArgs;

    impl InitOpenOrdersArgs {
        pub fn with_parsed_args<T>(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
            f: impl FnOnce(InitOpenOrdersArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            // Parse accounts.
            check_assert!(accounts.len() == 4 || accounts.len() == 5)?;
            #[rustfmt::skip]
            let &[
                ref open_orders_acc,
                ref owner_acc,
                ref market_acc,
                ref rent_acc,
            ] = array_ref![accounts, 0, 4];

            let oo_authority = (&accounts[4..])
                .first()
                .map(|acc| SignerAccount::new(acc))
                .transpose()?;

            // Dynamic sysvars don't work in unit tests.
            #[cfg(any(test, feature = "fuzz"))]
            let rent = Rent::from_account_info(rent_acc)?;
            #[cfg(not(any(test, feature = "fuzz")))]
            let rent = Rent::get()?;

            // Validate the accounts given are valid.
            let owner = SignerAccount::new(owner_acc)?;
            let market = Market::load(market_acc, program_id, false)?;

            // Perform open orders initialization.
            let _open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(owner.inner()),
                program_id,
                Some(rent),
                oo_authority,
            )?;

            // Invoke processor.
            f(InitOpenOrdersArgs)
        }
    }

    pub struct PruneArgs<'a> {
        pub order_book_state: OrderBookState<'a>,
        pub open_orders: &'a mut OpenOrders,
        pub open_orders_address: &'a Pubkey,
        pub event_q: EventQueue<'a>,
        pub limit: u16,
    }

    impl<'a> PruneArgs<'a> {
        pub fn with_parsed_args<T>(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
            limit: u16,
            f: impl FnOnce(PruneArgs) -> DexResult<T>,
        ) -> DexResult<T> {
            // Parse accounts.
            check_assert!(accounts.len() == 7)?;
            #[rustfmt::skip]
            let &[
                ref market_acc,
                ref bids_acc,
                ref asks_acc,
                ref prune_auth_acc,
                ref open_orders_acc,
                ref open_orders_owner_acc,
                ref event_q_acc,
            ] = array_ref![accounts, 0, 7];

            let _prune_authority = SignerAccount::new(prune_auth_acc)?;
            let mut market = Market::load(market_acc, program_id, false)?;
            check_assert!(market.prune_authority() == Some(prune_auth_acc.key))?;
            let open_orders_address = open_orders_acc.key;
            let mut open_orders = market.load_orders_mut(
                open_orders_acc,
                Some(open_orders_owner_acc),
                program_id,
                None,
                None,
            )?;
            let mut bids = market.load_bids_mut(bids_acc).or(check_unreachable!())?;
            let mut asks = market.load_asks_mut(asks_acc).or(check_unreachable!())?;
            let event_q = market.load_event_queue_mut(event_q_acc)?;
            let order_book_state = OrderBookState {
                bids: bids.deref_mut(),
                asks: asks.deref_mut(),
                market_state: market.deref_mut(),
            };

            let args = PruneArgs {
                order_book_state,
                open_orders_address,
                open_orders: open_orders.deref_mut(),
                event_q,
                limit,
            };
            f(args)
        }
    }
}

#[inline]
fn remove_slop<T: Pod>(bytes: &[u8]) -> &[T] {
    let slop = bytes.len() % size_of::<T>();
    let new_len = bytes.len() - slop;
    cast_slice(&bytes[..new_len])
}

#[inline]
fn remove_slop_mut<T: Pod>(bytes: &mut [u8]) -> &mut [T] {
    let slop = bytes.len() % size_of::<T>();
    let new_len = bytes.len() - slop;
    cast_slice_mut(&mut bytes[..new_len])
}

#[cfg_attr(not(feature = "program"), allow(unused))]
impl State {
    #[cfg(feature = "program")]
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> DexResult {
        let instruction = MarketInstruction::unpack(input).ok_or(ProgramError::InvalidArgument)?;
        match instruction {
            MarketInstruction::InitializeMarket(ref inner) => Self::process_initialize_market(
                account_parser::InitializeMarketArgs::new(program_id, inner, accounts)?,
            )?,
            MarketInstruction::NewOrder(_inner) => {
                unimplemented!()
            }
            MarketInstruction::NewOrderV2(_inner) => {
                unimplemented!()
            }
            MarketInstruction::NewOrderV3(ref inner) => {
                account_parser::NewOrderV3Args::with_parsed_args(
                    program_id,
                    inner,
                    accounts,
                    Self::process_new_order_v3,
                )?
            }
            MarketInstruction::ReplaceOrderByClientId(instruction) => {
                account_parser::ReplaceOrdersByClientIdsArgs::with_parsed_args(
                    program_id,
                    accounts,
                    &vec![instruction],
                    Self::process_replace_orders_by_client_ids,
                )?
            }
            MarketInstruction::ReplaceOrdersByClientIds(ref instructions) => {
                account_parser::ReplaceOrdersByClientIdsArgs::with_parsed_args(
                    program_id,
                    accounts,
                    instructions,
                    Self::process_replace_orders_by_client_ids,
                )?
            }
            MarketInstruction::MatchOrders(_limit) => {}
            MarketInstruction::ConsumeEvents(limit) => {
                account_parser::ConsumeEventsArgs::with_parsed_args(
                    program_id,
                    accounts,
                    limit,
                    Self::process_consume_events,
                )?
            }
            MarketInstruction::ConsumeEventsPermissioned(limit) => {
                account_parser::ConsumeEventsArgs::with_parsed_args_permissioned(
                    program_id,
                    accounts,
                    limit,
                    Self::process_consume_events,
                )?
            }
            MarketInstruction::CancelOrder(_inner) => {
                unimplemented!()
            }
            MarketInstruction::CancelOrderV2(ref inner) => {
                account_parser::CancelOrderV2Args::with_parsed_args(
                    program_id,
                    accounts,
                    inner,
                    Self::process_cancel_order_v2,
                )?
            }
            MarketInstruction::SettleFunds => account_parser::SettleFundsArgs::with_parsed_args(
                program_id,
                accounts,
                Self::process_settle_funds,
            )?,
            MarketInstruction::CancelOrderByClientId(_client_id) => {
                unimplemented!()
            }
            MarketInstruction::CancelOrderByClientIdV2(client_id) => {
                account_parser::CancelOrderByClientIdV2Args::with_parsed_args(
                    program_id,
                    accounts,
                    client_id,
                    Self::process_cancel_order_by_client_id_v2,
                )?
            }
            MarketInstruction::CancelOrdersByClientIds(client_ids) => {
                account_parser::CancelOrdersByClientIdsArgs::with_parsed_args(
                    program_id,
                    accounts,
                    client_ids,
                    Self::process_cancel_orders_by_client_ids,
                )?
            }
            MarketInstruction::DisableMarket => {
                account_parser::DisableMarketArgs::with_parsed_args(
                    program_id,
                    accounts,
                    Self::process_disable_market,
                )?
            }
            MarketInstruction::SweepFees => account_parser::SweepFeesArgs::with_parsed_args(
                program_id,
                accounts,
                Self::process_sweep_fees,
            )?,
            MarketInstruction::SendTake(ref inner) => {
                account_parser::SendTakeArgs::with_parsed_args(
                    program_id,
                    inner,
                    accounts,
                    Self::process_send_take,
                )?
            }
            MarketInstruction::CloseOpenOrders => {
                account_parser::CloseOpenOrdersArgs::with_parsed_args(
                    program_id,
                    accounts,
                    Self::process_close_open_orders,
                )?
            }
            MarketInstruction::InitOpenOrders => {
                account_parser::InitOpenOrdersArgs::with_parsed_args(
                    program_id,
                    accounts,
                    Self::process_init_open_orders,
                )?
            }
            MarketInstruction::Prune(limit) => account_parser::PruneArgs::with_parsed_args(
                program_id,
                accounts,
                limit,
                Self::process_prune,
            )?,
        };
        Ok(())
    }

    #[cfg(feature = "program")]
    fn process_send_take(_args: account_parser::SendTakeArgs) -> DexResult {
        unimplemented!()
    }

    fn process_prune(args: account_parser::PruneArgs) -> DexResult {
        let account_parser::PruneArgs {
            mut order_book_state,
            open_orders,
            open_orders_address,
            mut event_q,
            limit,
        } = args;
        let open_orders_addr_bytes = open_orders_address.to_aligned_bytes();
        let (bids_removed, asks_removed) =
            order_book_state.remove_all(open_orders_addr_bytes, limit)?;

        solana_program::msg!(
            "Pruned {:?} bids and {:?} asks",
            bids_removed.len(),
            asks_removed.len()
        );

        for bid in bids_removed {
            let order_id = open_orders.orders[bid.owner_slot() as usize];
            order_book_state.cancel_leaf_node(
                bid,
                Side::Bid,
                open_orders,
                open_orders_addr_bytes,
                order_id,
                &mut event_q,
            )?;
        }

        for ask in asks_removed {
            let order_id = open_orders.orders[ask.owner_slot() as usize];
            order_book_state.cancel_leaf_node(
                ask,
                Side::Ask,
                open_orders,
                open_orders_addr_bytes,
                order_id,
                &mut event_q,
            )?;
        }

        Ok(())
    }

    fn process_init_open_orders(_args: account_parser::InitOpenOrdersArgs) -> DexResult {
        Ok(())
    }

    fn process_close_open_orders(args: account_parser::CloseOpenOrdersArgs) -> DexResult {
        let account_parser::CloseOpenOrdersArgs {
            open_orders,
            open_orders_acc,
            dest_acc,
        } = args;

        // Transfer all lamports to the destination.
        let dest_starting_lamports = dest_acc.lamports();
        **dest_acc.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(open_orders_acc.lamports())
            .unwrap();
        **open_orders_acc.lamports.borrow_mut() = 0;

        // Mark the account as closed to prevent it from being used before
        // garbage collection.
        open_orders.account_flags = AccountFlag::Closed as u64;

        Ok(())
    }

    #[cfg(feature = "program")]
    fn process_settle_funds(args: account_parser::SettleFundsArgs) -> DexResult {
        let account_parser::SettleFundsArgs {
            mut market,
            mut open_orders,
            coin_vault,
            pc_vault,
            coin_wallet,
            pc_wallet,
            vault_signer,
            spl_token_program,
            referrer,
        } = args;

        let native_coin_amount = open_orders.native_coin_free;
        let native_pc_amount = open_orders.native_pc_free;

        market.coin_deposits_total -= native_coin_amount;
        market.pc_deposits_total -= native_pc_amount;

        open_orders.native_coin_free = 0;
        open_orders.native_pc_free = 0;

        open_orders.native_coin_total = open_orders
            .native_coin_total
            .checked_sub(native_coin_amount)
            .unwrap();
        open_orders.native_pc_total = open_orders
            .native_pc_total
            .checked_sub(native_pc_amount)
            .unwrap();

        let token_infos: [(
            u64,
            account_parser::TokenAccount,
            account_parser::TokenAccount,
        ); 2] = [
            (
                native_coin_amount,
                coin_wallet.token_account(),
                coin_vault.token_account(),
            ),
            (
                native_pc_amount,
                pc_wallet.token_account(),
                pc_vault.token_account(),
            ),
        ];

        let nonce = market.vault_signer_nonce;
        let market_pubkey = market.pubkey();
        let vault_signer_seeds = gen_vault_signer_seeds(&nonce, &market_pubkey);

        for &(token_amount, wallet_account, vault) in token_infos.iter() {
            send_from_vault(
                token_amount,
                wallet_account,
                vault,
                spl_token_program,
                vault_signer,
                &vault_signer_seeds,
            )?;
        }

        match referrer {
            Some(referrer_pc_wallet) if open_orders.referrer_rebates_accrued > 0 => {
                send_from_vault(
                    open_orders.referrer_rebates_accrued,
                    referrer_pc_wallet.token_account(),
                    pc_vault.token_account(),
                    spl_token_program,
                    vault_signer,
                    &vault_signer_seeds,
                )?;
            }
            _ => {
                market.pc_fees_accrued += open_orders.referrer_rebates_accrued;
            }
        };
        market.referrer_rebates_accrued -= open_orders.referrer_rebates_accrued;
        open_orders.referrer_rebates_accrued = 0;

        Ok(())
    }

    fn process_cancel_order_by_client_id_v2(
        args: account_parser::CancelOrderByClientIdV2Args,
    ) -> DexResult {
        let account_parser::CancelOrderByClientIdV2Args {
            client_order_id,
            open_orders_address,
            open_orders,
            open_orders_signer: _,

            mut order_book_state,
            mut event_q,
        } = args;

        let (_, order_id, side) = open_orders
            .orders_with_client_ids()
            .find(|entry| client_order_id == entry.0)
            .ok_or(DexErrorCode::ClientIdNotFound)?;
        order_book_state.cancel_order_v2(
            side,
            open_orders_address,
            open_orders,
            order_id,
            &mut event_q,
        )
    }

    fn process_cancel_orders_by_client_ids(
        args: account_parser::CancelOrdersByClientIdsArgs,
    ) -> DexResult {
        let account_parser::CancelOrdersByClientIdsArgs {
            client_order_ids,
            open_orders_address,
            open_orders,
            open_orders_signer: _,

            mut order_book_state,
            mut event_q,
        } = args;

        let mut orders = Vec::new();
        for (client_order_id, order_id, side) in open_orders.orders_with_client_ids() {
            if client_order_ids.contains(&client_order_id) {
                orders.push((order_id, side));
            }
        }

        for (order_id, side) in orders {
            if let Err(err) = order_book_state.cancel_order_v2(
                side,
                open_orders_address,
                open_orders,
                order_id,
                &mut event_q,
            ) {
                // Cancel returns OrderNotFound when the order isn't in bids or
                // asks. In this case, no Serum state is modified so it is safe
                // to continue. Any other type of error could be accompanied by
                // a partial state change, so we must error. For example, if the
                // event queue is full, we should abort to prevent the cancelled
                // order from being removed from bids/asks.
                match err {
                    DexError::ErrorCode(code) => {
                        if code == DexErrorCode::OrderNotFound {
                            continue;
                        }
                        return Err(err);
                    }
                    _ => return Err(err),
                }
            }
        }

        Ok(())
    }

    fn process_cancel_order_v2(args: account_parser::CancelOrderV2Args) -> DexResult {
        let account_parser::CancelOrderV2Args {
            instruction: &CancelOrderInstructionV2 { side, order_id },

            open_orders_address,
            open_orders,
            open_orders_signer: _,

            mut order_book_state,
            mut event_q,
        } = args;

        order_book_state.cancel_order_v2(
            side,
            open_orders_address,
            open_orders,
            order_id,
            &mut event_q,
        )
    }

    fn process_consume_events(args: account_parser::ConsumeEventsArgs) -> DexResult {
        let account_parser::ConsumeEventsArgs {
            limit,
            program_id,
            open_orders_accounts,
            market,
            mut event_q,
        } = args;

        for _i in 0u16..limit {
            let event = match event_q.peek_front() {
                None => break,
                Some(e) => e,
            };

            let view = event.as_view()?;
            let owner: [u64; 4] = event.owner;
            let owner_index: Result<usize, usize> = open_orders_accounts
                .binary_search_by_key(&owner, |account_info| account_info.key.to_aligned_bytes());
            let mut open_orders: RefMut<OpenOrders> = match owner_index {
                Err(_) => break,
                Ok(i) => market.load_orders_mut(
                    &open_orders_accounts[i],
                    None,
                    program_id,
                    None,
                    None,
                )?,
            };

            check_assert!(event.owner_slot < 128)?;
            check_assert_eq!(&open_orders.slot_side(event.owner_slot), &Some(view.side()))?;
            check_assert_eq!(
                open_orders.orders[event.owner_slot as usize],
                event.order_id
            )?;

            match event.as_view()? {
                EventView::Fill {
                    side,
                    maker,
                    native_qty_paid,
                    native_qty_received,
                    native_fee_or_rebate,
                    fee_tier,
                    order_id,
                    owner,
                    owner_slot,
                    client_order_id,
                } => {
                    match side {
                        Side::Bid if maker => {
                            open_orders.native_pc_total -= native_qty_paid;
                            open_orders.native_coin_total += native_qty_received;
                            open_orders.native_coin_free += native_qty_received;
                            open_orders.native_pc_free += native_fee_or_rebate;
                        }
                        Side::Ask if maker => {
                            open_orders.native_coin_total -= native_qty_paid;
                            open_orders.native_pc_total += native_qty_received;
                            open_orders.native_pc_free += native_qty_received;
                        }
                        _ => (),
                    };

                    let referrer_rebate = if !maker {
                        let referrer_rebate = fees::referrer_rebate(native_fee_or_rebate);
                        open_orders.referrer_rebates_accrued += referrer_rebate;
                        Some(referrer_rebate)
                    } else {
                        None
                    };

                    if let Some(client_id) = client_order_id {
                        debug_assert_eq!(
                            client_id.get(),
                            identity(open_orders.client_order_ids[owner_slot as usize])
                        );
                    }

                    let open_orders_pk = Pubkey::new(cast_slice(&identity(owner) as &[_]));
                    let open_orders_owner_pk =
                        Pubkey::new(cast_slice(&identity(open_orders.owner) as &[_]));
                    emit!(FillEventLog {
                        market: market.pubkey(),
                        bid: match side {
                            Side::Bid => true,
                            Side::Ask => false,
                        },
                        maker,
                        native_qty_paid,
                        native_qty_received,
                        native_fee_or_rebate,
                        order_id,
                        open_orders: open_orders_pk,
                        open_orders_owner: open_orders_owner_pk,
                        owner_slot,
                        fee_tier: fee_tier as u8,
                        client_order_id: client_order_id.map(|i| i.get()),
                        referrer_rebate
                    });

                    emit!(OpenOrdersBalanceLog {
                        open_orders: open_orders_pk,
                        open_orders_owner: open_orders_owner_pk,
                        market: market.pubkey(),
                        native_pc_total: open_orders.native_pc_total,
                        native_coin_total: open_orders.native_coin_total,
                        native_coin_free: open_orders.native_coin_free,
                        native_pc_free: open_orders.native_pc_free,
                        referrer_rebates_accrued: open_orders.referrer_rebates_accrued,
                    })
                }
                EventView::Out {
                    side,
                    release_funds,
                    native_qty_unlocked,
                    native_qty_still_locked,
                    order_id: _,
                    owner: _,
                    owner_slot,
                    client_order_id,
                } => {
                    let fully_out = native_qty_still_locked == 0;

                    match side {
                        Side::Bid => {
                            if release_funds {
                                open_orders.native_pc_free += native_qty_unlocked;
                            }
                            check_assert!(
                                open_orders.native_pc_free <= open_orders.native_pc_total
                            )?;
                        }
                        Side::Ask => {
                            if release_funds {
                                open_orders.native_coin_free += native_qty_unlocked;
                            }
                            check_assert!(
                                open_orders.native_coin_free <= open_orders.native_coin_total
                            )?;
                        }
                    };
                    if let Some(client_id) = client_order_id {
                        debug_assert_eq!(
                            client_id.get(),
                            identity(open_orders.client_order_ids[owner_slot as usize])
                        );
                    }
                    if fully_out {
                        open_orders.remove_order(owner_slot)?;
                    }
                }
            };

            event_q
                .pop_front()
                .map_err(|()| DexErrorCode::ConsumeEventsQueueFailure)?;
        }
        Ok(())
    }

    #[cfg(feature = "program")]
    fn process_new_order_v3(args: account_parser::NewOrderV3Args) -> DexResult {
        let account_parser::NewOrderV3Args {
            instruction,
            mut order_book_state,
            mut open_orders,
            open_orders_address,
            mut req_q,
            mut event_q,
            payer,
            owner,
            coin_vault,
            pc_vault,
            spl_token_program,
            fee_tier,
        } = args;

        if instruction.max_ts < i64::MAX {
            // Dynamic sysvars don't work in unit tests.
            #[cfg(any(test, feature = "fuzz"))]
            let cur_ts = 1_650_000_000;
            #[cfg(not(any(test, feature = "fuzz")))]
            let cur_ts = Clock::get()?.unix_timestamp;
            if cur_ts > instruction.max_ts {
                return Err(DexErrorCode::OrderMaxTimestampExceeded.into());
            }
        }

        let open_orders_mut = open_orders.deref_mut();

        check_assert_eq!(req_q.header.count(), 0)?;

        let deposit_amount;
        let deposit_vault;

        let native_pc_qty_locked;
        match instruction.side {
            Side::Bid => {
                let lock_qty_native = instruction.max_native_pc_qty_including_fees;
                native_pc_qty_locked = Some(lock_qty_native);
                let free_qty_to_lock = lock_qty_native.get().min(open_orders_mut.native_pc_free);
                deposit_amount = lock_qty_native.get() - free_qty_to_lock;
                deposit_vault = pc_vault.token_account();
                if payer.balance()? < deposit_amount {
                    return Err(DexErrorCode::InsufficientFunds.into());
                }
                open_orders_mut.lock_free_pc(free_qty_to_lock);
                open_orders_mut.credit_locked_pc(deposit_amount);
                order_book_state.market_state.pc_deposits_total = order_book_state
                    .market_state
                    .pc_deposits_total
                    .checked_add(deposit_amount)
                    .unwrap();
            }
            Side::Ask => {
                native_pc_qty_locked = None;
                let lock_qty_native = instruction
                    .max_coin_qty
                    .get()
                    .checked_mul(order_book_state.market_state.coin_lot_size)
                    .ok_or(DexErrorCode::InsufficientFunds)?;
                let free_qty_to_lock = lock_qty_native.min(open_orders_mut.native_coin_free);
                deposit_amount = lock_qty_native - free_qty_to_lock;
                deposit_vault = coin_vault.token_account();
                if payer.balance()? < deposit_amount {
                    return Err(DexErrorCode::InsufficientFunds.into());
                }
                open_orders_mut.lock_free_coin(free_qty_to_lock);
                open_orders_mut.credit_locked_coin(deposit_amount);
                order_book_state.market_state.coin_deposits_total = order_book_state
                    .market_state
                    .coin_deposits_total
                    .checked_add(deposit_amount)
                    .unwrap();
            }
        };

        let order_id = req_q.gen_order_id(instruction.limit_price.get(), instruction.side);
        let owner_slot = open_orders_mut.add_order(order_id, instruction.side)?;
        open_orders_mut.client_order_ids[owner_slot as usize] = instruction.client_order_id;

        let mut proceeds = RequestProceeds::zero();

        let request = RequestView::NewOrder {
            side: instruction.side,
            order_type: instruction.order_type,
            order_id,
            fee_tier,
            self_trade_behavior: instruction.self_trade_behavior,
            owner: open_orders_address,
            owner_slot,
            max_coin_qty: instruction.max_coin_qty,
            native_pc_qty_locked,
            client_order_id: NonZeroU64::new(instruction.client_order_id),
        };
        let mut limit = instruction.limit;
        let unfilled_portion = order_book_state.process_orderbook_request(
            &request,
            &mut event_q,
            &mut proceeds,
            &mut limit,
        )?;

        check_assert!(unfilled_portion.is_none())?;

        {
            let coin_lot_size = order_book_state.market_state.coin_lot_size;

            let RequestProceeds {
                coin_unlocked,
                coin_credit,

                native_pc_unlocked,
                native_pc_credit,

                coin_debit,
                native_pc_debit,
            } = proceeds;

            let native_coin_unlocked = coin_unlocked.checked_mul(coin_lot_size).unwrap();
            let native_coin_credit = coin_credit.checked_mul(coin_lot_size).unwrap();
            let native_coin_debit = coin_debit.checked_mul(coin_lot_size).unwrap();

            open_orders_mut.credit_locked_coin(native_coin_credit);
            open_orders_mut.unlock_coin(native_coin_credit);
            open_orders_mut.unlock_coin(native_coin_unlocked);

            open_orders_mut.credit_locked_pc(native_pc_credit);
            open_orders_mut.unlock_pc(native_pc_credit);
            open_orders_mut.unlock_pc(native_pc_unlocked);

            open_orders_mut.native_coin_total = open_orders_mut
                .native_coin_total
                .checked_sub(native_coin_debit)
                .unwrap();
            open_orders_mut.native_pc_total = open_orders_mut
                .native_pc_total
                .checked_sub(native_pc_debit)
                .unwrap();
            check_assert!(open_orders_mut.native_coin_free <= open_orders_mut.native_coin_total)?;
            check_assert!(open_orders_mut.native_pc_free <= open_orders_mut.native_pc_total)?;
        }

        // Drop the open orders account in the event that it is the owner
        // of itself, which may happen if the account is a PDA.
        //
        // `invoke_spl_token` will try to borrow the account info refcell,
        // which would cause an error (as there would be two borrows while
        // one of them is mutable).

        use solana_program::clock::Clock;

        drop(open_orders);

        if deposit_amount != 0 {
            let balance_before = deposit_vault.balance()?;
            let deposit_instruction = spl_token::instruction::transfer(
                &spl_token::ID,
                payer.inner().key,
                deposit_vault.inner().key,
                owner.inner().key,
                &[],
                deposit_amount,
            )
            .unwrap();
            invoke_spl_token(
                &deposit_instruction,
                &[
                    payer.inner().clone(),
                    deposit_vault.inner().clone(),
                    owner.inner().clone(),
                    spl_token_program.inner().clone(),
                ],
                &[],
            )
            .map_err(|err| match err {
                ProgramError::Custom(i) => match TokenError::from_u32(i) {
                    Some(TokenError::InsufficientFunds) => DexErrorCode::InsufficientFunds,
                    _ => DexErrorCode::TransferFailed,
                },
                _ => DexErrorCode::TransferFailed,
            })?;
            let balance_after = deposit_vault.balance()?;
            let balance_change = balance_after.checked_sub(balance_before);
            check_assert_eq!(Some(deposit_amount), balance_change)?;
        }

        Ok(())
    }

    fn process_replace_orders_by_client_ids(
        args: account_parser::ReplaceOrdersByClientIdsArgs,
    ) -> DexResult {
        let account_parser::ReplaceOrdersByClientIdsArgs {
            program_id,
            cancel_accounts,
            client_order_ids,
            instructions,
            accounts,
        } = args;

        account_parser::CancelOrdersByClientIdsArgs::with_parsed_args(
            program_id,
            &cancel_accounts,
            client_order_ids,
            Self::process_cancel_orders_by_client_ids,
        )?;

        for instruction in instructions {
            account_parser::NewOrderV3Args::with_parsed_args(
                program_id,
                instruction,
                accounts,
                Self::process_new_order_v3,
            )?;
        }

        Ok(())
    }

    fn process_disable_market(args: account_parser::DisableMarketArgs) -> DexResult {
        let account_parser::DisableMarketArgs {
            market,
            authorization: _,
        } = args;
        market.account_flags = market.account_flags | (AccountFlag::Disabled as u64);
        Ok(())
    }

    #[cfg(feature = "program")]
    fn process_sweep_fees(args: account_parser::SweepFeesArgs) -> DexResult {
        let account_parser::SweepFeesArgs {
            mut market,
            pc_vault,
            fee_receiver,
            vault_signer,
            spl_token_program,
            authorization: _,
        } = args;
        let token_amount = market.pc_fees_accrued;
        market.pc_fees_accrued = 0;

        let nonce = market.vault_signer_nonce;
        let market_pubkey = market.pubkey();
        let vault_signer_seeds = gen_vault_signer_seeds(&nonce, &market_pubkey);
        send_from_vault(
            token_amount,
            fee_receiver.token_account(),
            pc_vault.token_account(),
            spl_token_program,
            vault_signer,
            &vault_signer_seeds,
        )
    }

    fn process_initialize_market(args: account_parser::InitializeMarketArgs) -> DexResult {
        let &InitializeMarketInstruction {
            coin_lot_size,
            pc_lot_size,
            fee_rate_bps,
            vault_signer_nonce,
            pc_dust_threshold,
        } = args.instruction;

        let market = args.get_market();
        let req_q = args.get_req_q();
        let event_q = args.get_event_q();
        let bids = args.get_bids();
        let asks = args.get_asks();
        let coin_vault = args.coin_vault_and_mint.get_account().inner();
        let coin_mint = args.coin_vault_and_mint.get_mint().inner();
        let pc_vault = args.pc_vault_and_mint.get_account().inner();
        let pc_mint = args.pc_vault_and_mint.get_mint().inner();
        let market_authority = args.market_authority;
        let prune_authority = args.prune_authority;
        let consume_events_authority = args.consume_events_authority;

        // initialize request queue
        let mut rq_data = req_q.try_borrow_mut_data()?;
        const RQ_HEADER_WORDS: usize = size_of::<RequestQueueHeader>() / size_of::<u64>();
        let rq_view = init_account_padding(&mut rq_data)?;
        let (rq_hdr_array, rq_buf_words) = mut_array_refs![rq_view, RQ_HEADER_WORDS; .. ;];
        let rq_buf: &[Request] = remove_slop(cast_slice(rq_buf_words));
        if rq_buf.is_empty() {
            Err(DexErrorCode::RequestQueueEmpty)?
        }
        let rq_hdr: &mut RequestQueueHeader =
            try_cast_mut(rq_hdr_array).or(check_unreachable!())?;
        *rq_hdr = RequestQueueHeader {
            account_flags: (AccountFlag::Initialized | AccountFlag::RequestQueue).bits(),
            head: 0,
            count: 0,
            next_seq_num: 0,
        };
        // initialize event queue
        let mut eq_data = event_q.try_borrow_mut_data().unwrap();
        const EQ_HEADER_WORDS: usize = size_of::<EventQueueHeader>() / size_of::<u64>();
        let eq_view = init_account_padding(&mut eq_data)?;
        check_assert!(eq_view.len() > EQ_HEADER_WORDS)?;
        let (eq_hdr_array, eq_buf_words) = mut_array_refs![eq_view, EQ_HEADER_WORDS; .. ;];
        let eq_buf: &[Event] = remove_slop(cast_slice(eq_buf_words));
        if eq_buf.len() < 128 {
            Err(DexErrorCode::EventQueueTooSmall)?
        }
        let eq_hdr: &mut EventQueueHeader = try_cast_mut(eq_hdr_array).or(check_unreachable!())?;
        *eq_hdr = EventQueueHeader {
            account_flags: (AccountFlag::Initialized | AccountFlag::EventQueue).bits(),
            head: 0,
            count: 0,
            seq_num: 0,
        };
        // initialize orderbook storage
        for (flag, account) in &[(AccountFlag::Bids, bids), (AccountFlag::Asks, asks)] {
            let mut ob_data = account.try_borrow_mut_data().unwrap();
            let ob_view = init_account_padding(&mut ob_data)?;
            const OB_HEADER_WORDS: usize = size_of::<OrderBookStateHeader>() / size_of::<u64>();
            check_assert!(ob_view.len() > OB_HEADER_WORDS)?;
            let (hdr_array, slab_words) = mut_array_refs![ob_view, OB_HEADER_WORDS; .. ;];
            let ob_hdr: &mut OrderBookStateHeader =
                try_cast_mut(hdr_array).or(check_unreachable!())?;
            *ob_hdr = OrderBookStateHeader {
                account_flags: (AccountFlag::Initialized | *flag).bits(),
            };
            let slab = Slab::new(cast_slice_mut(slab_words));
            slab.assert_minimum_capacity(100)?;
        }
        // initialize market
        let mut market_data = market.try_borrow_mut_data()?;
        let market_view = init_account_padding(&mut market_data)?;
        let mut account_flags = AccountFlag::Initialized | AccountFlag::Market;
        if market_authority.is_some() {
            account_flags |= AccountFlag::Permissioned;
            if consume_events_authority.is_some() {
                account_flags |= AccountFlag::CrankAuthorityRequired;
            }
        }
        let market_state = MarketState {
            coin_lot_size,
            pc_lot_size,
            own_address: market.key.to_aligned_bytes(),
            account_flags: account_flags.bits(),

            coin_mint: coin_mint.key.to_aligned_bytes(),
            coin_vault: coin_vault.key.to_aligned_bytes(),
            coin_deposits_total: 0,
            coin_fees_accrued: 0,

            req_q: req_q.key.to_aligned_bytes(),
            event_q: event_q.key.to_aligned_bytes(),
            bids: bids.key.to_aligned_bytes(),
            asks: asks.key.to_aligned_bytes(),

            pc_mint: pc_mint.key.to_aligned_bytes(),
            pc_vault: pc_vault.key.to_aligned_bytes(),
            pc_deposits_total: 0,
            pc_fees_accrued: 0,
            vault_signer_nonce,

            pc_dust_threshold,
            fee_rate_bps: fee_rate_bps as u64,
            referrer_rebates_accrued: 0,
        };
        match market_authority {
            None => {
                let market_hdr: &mut MarketState =
                    try_from_bytes_mut(cast_slice_mut(market_view)).or(check_unreachable!())?;
                *market_hdr = market_state;
            }
            Some(oo_auth) => {
                let market_hdr: &mut MarketStateV2 =
                    try_from_bytes_mut(cast_slice_mut(market_view)).or(check_unreachable!())?;
                market_hdr.inner = market_state;
                market_hdr.open_orders_authority = *oo_auth.key;
                market_hdr.prune_authority =
                    prune_authority.map(|p| *p.key).unwrap_or(Pubkey::default());

                if let Some(consume_events_authority) = consume_events_authority {
                    market_hdr.consume_events_authority = *consume_events_authority.key;
                }
            }
        }
        Ok(())
    }
}