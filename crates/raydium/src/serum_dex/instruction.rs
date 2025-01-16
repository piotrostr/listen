use crate::serum_dex::{
    error::DexError,
    matching::{OrderType, Side},
};
use bytemuck::cast;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
    sysvar::rent,
};
use std::convert::TryInto;

use arrayref::{array_ref, array_refs};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::num::NonZeroU64;

pub const RENT_PROGRAM: Pubkey =
    pubkey!("SysvarRent111111111111111111111111111111111");

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct InitializeMarketInstruction {
    // In the matching engine, all prices and balances are integers.
    // This only works if the smallest representable quantity of the coin
    // is at least a few orders of magnitude larger than the smallest representable
    // quantity of the price currency. The internal representation also relies on
    // on the assumption that every order will have a (quantity x price) value that
    // fits into a u64.
    //
    // If these assumptions are problematic, rejigger the lot sizes.
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
    pub fee_rate_bps: u16,
    pub vault_signer_nonce: u64,
    pub pc_dust_threshold: u64,
}

#[derive(
    PartialEq,
    Eq,
    Copy,
    Clone,
    Debug,
    TryFromPrimitive,
    IntoPrimitive,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum SelfTradeBehavior {
    DecrementTake = 0,
    CancelProvide = 1,
    AbortTransaction = 2,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct SendTakeInstruction {
    pub side: Side,

    pub limit_price: NonZeroU64,

    pub max_coin_qty: NonZeroU64,

    pub max_native_pc_qty_including_fees: NonZeroU64,

    pub min_coin_qty: u64,
    pub min_native_pc_qty: u64,

    pub limit: u16,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderInstructionV3 {
    pub side: Side,

    pub limit_price: NonZeroU64,

    pub max_coin_qty: NonZeroU64,

    pub max_native_pc_qty_including_fees: NonZeroU64,

    pub self_trade_behavior: SelfTradeBehavior,

    pub order_type: OrderType,
    pub client_order_id: u64,
    pub limit: u16,
    pub max_ts: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderInstructionV2 {
    pub side: Side,

    pub limit_price: NonZeroU64,

    pub max_qty: NonZeroU64,
    pub order_type: OrderType,
    pub client_id: u64,
    pub self_trade_behavior: SelfTradeBehavior,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderInstructionV1 {
    pub side: Side,

    pub limit_price: NonZeroU64,

    pub max_qty: NonZeroU64,
    pub order_type: OrderType,
    pub client_id: u64,
}

impl NewOrderInstructionV1 {
    pub fn add_self_trade_behavior(
        self,
        self_trade_behavior: SelfTradeBehavior,
    ) -> NewOrderInstructionV2 {
        let NewOrderInstructionV1 {
            side,
            limit_price,
            max_qty,
            order_type,
            client_id,
        } = self;
        NewOrderInstructionV2 {
            side,
            limit_price,
            max_qty,
            order_type,
            client_id,
            self_trade_behavior,
        }
    }
}

impl SendTakeInstruction {
    fn unpack(data: &[u8; 46]) -> Option<Self> {
        let (
            &side_arr,
            &price_arr,
            &max_coin_qty_arr,
            &max_native_pc_qty_arr,
            &min_coin_qty_arr,
            &min_native_pc_qty_arr,
            &limit_arr,
        ) = array_refs![data, 4, 8, 8, 8, 8, 8, 2];

        let side = Side::try_from_primitive(
            u32::from_le_bytes(side_arr).try_into().ok()?,
        )
        .ok()?;
        let limit_price = NonZeroU64::new(u64::from_le_bytes(price_arr))?;
        let max_coin_qty =
            NonZeroU64::new(u64::from_le_bytes(max_coin_qty_arr))?;
        let max_native_pc_qty_including_fees =
            NonZeroU64::new(u64::from_le_bytes(max_native_pc_qty_arr))?;
        let min_coin_qty = u64::from_le_bytes(min_coin_qty_arr);
        let min_native_pc_qty = u64::from_le_bytes(min_native_pc_qty_arr);
        let limit = u16::from_le_bytes(limit_arr);

        Some(SendTakeInstruction {
            side,
            limit_price,
            max_coin_qty,
            max_native_pc_qty_including_fees,
            min_coin_qty,
            min_native_pc_qty,
            limit,
        })
    }
}

impl NewOrderInstructionV3 {
    fn unpack(data: &[u8; 54]) -> Option<Self> {
        let (
            &side_arr,
            &price_arr,
            &max_coin_qty_arr,
            &max_native_pc_qty_arr,
            &self_trade_behavior_arr,
            &otype_arr,
            &client_order_id_bytes,
            &limit_arr,
            &max_ts,
        ) = array_refs![data, 4, 8, 8, 8, 4, 4, 8, 2, 8];

        let side = Side::try_from_primitive(
            u32::from_le_bytes(side_arr).try_into().ok()?,
        )
        .ok()?;
        let limit_price = NonZeroU64::new(u64::from_le_bytes(price_arr))?;
        let max_coin_qty =
            NonZeroU64::new(u64::from_le_bytes(max_coin_qty_arr))?;
        let max_native_pc_qty_including_fees =
            NonZeroU64::new(u64::from_le_bytes(max_native_pc_qty_arr))?;
        let self_trade_behavior = SelfTradeBehavior::try_from_primitive(
            u32::from_le_bytes(self_trade_behavior_arr)
                .try_into()
                .ok()?,
        )
        .ok()?;
        let order_type = OrderType::try_from_primitive(
            u32::from_le_bytes(otype_arr).try_into().ok()?,
        )
        .ok()?;
        let client_order_id = u64::from_le_bytes(client_order_id_bytes);
        let limit = u16::from_le_bytes(limit_arr);
        let max_ts = i64::from_le_bytes(max_ts);

        Some(NewOrderInstructionV3 {
            side,
            limit_price,
            max_coin_qty,
            max_native_pc_qty_including_fees,
            self_trade_behavior,
            order_type,
            client_order_id,
            limit,
            max_ts,
        })
    }
}

impl NewOrderInstructionV1 {
    fn unpack(data: &[u8; 32]) -> Option<Self> {
        let (
            &side_arr,
            &price_arr,
            &max_qty_arr,
            &otype_arr,
            &client_id_bytes,
        ) = array_refs![data, 4, 8, 8, 4, 8];
        let client_id = u64::from_le_bytes(client_id_bytes);
        let side = match u32::from_le_bytes(side_arr) {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => return None,
        };
        let limit_price = NonZeroU64::new(u64::from_le_bytes(price_arr))?;
        let max_qty = NonZeroU64::new(u64::from_le_bytes(max_qty_arr))?;
        let order_type = match u32::from_le_bytes(otype_arr) {
            0 => OrderType::Limit,
            1 => OrderType::ImmediateOrCancel,
            2 => OrderType::PostOnly,
            _ => return None,
        };
        Some(NewOrderInstructionV1 {
            side,
            limit_price,
            max_qty,
            order_type,
            client_id,
        })
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CancelOrderInstructionV2 {
    pub side: Side,
    pub order_id: u128,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct CancelOrderInstruction {
    pub side: Side,
    pub order_id: u128,
    pub owner: [u64; 4], // Unused
    pub owner_slot: u8,
}

impl CancelOrderInstructionV2 {
    fn unpack(data: &[u8; 20]) -> Option<Self> {
        let (&side_arr, &oid_arr) = array_refs![data, 4, 16];
        let side = Side::try_from_primitive(
            u32::from_le_bytes(side_arr).try_into().ok()?,
        )
        .ok()?;
        let order_id = u128::from_le_bytes(oid_arr);
        Some(CancelOrderInstructionV2 { side, order_id })
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum MarketInstruction {
    /// 0. `[writable]` the market to initialize
    /// 1. `[writable]` zeroed out request queue
    /// 2. `[writable]` zeroed out event queue
    /// 3. `[writable]` zeroed out bids
    /// 4. `[writable]` zeroed out asks
    /// 5. `[writable]` spl-token account for the coin currency
    /// 6. `[writable]` spl-token account for the price currency
    /// 7. `[]` coin currency Mint
    /// 8. `[]` price currency Mint
    /// 9. `[]` the rent sysvar
    /// 10. `[]` open orders market authority (optional)
    /// 11. `[]` prune authority (optional, requires open orders market authority)
    /// 12. `[]` crank authority (optional, requires prune authority)
    InitializeMarket(InitializeMarketInstruction),
    /// 0. `[writable]` the market
    /// 1. `[writable]` the OpenOrders account to use
    /// 2. `[writable]` the request queue
    /// 3. `[writable]` the (coin or price currency) account paying for the order
    /// 4. `[signer]` owner of the OpenOrders account
    /// 5. `[writable]` coin vault
    /// 6. `[writable]` pc vault
    /// 7. `[]` spl token program
    /// 8. `[]` the rent sysvar
    /// 9. `[]` (optional) the (M)SRM account used for fee discounts
    NewOrder(NewOrderInstructionV1),
    /// 0. `[writable]` market
    /// 1. `[writable]` req_q
    /// 2. `[writable]` event_q
    /// 3. `[writable]` bids
    /// 4. `[writable]` asks
    MatchOrders(u16),
    /// ... `[writable]` OpenOrders
    /// accounts.len() - 4 `[writable]` market
    /// accounts.len() - 3 `[writable]` event queue
    /// accounts.len() - 2 `[]`
    /// accounts.len() - 1 `[]`
    ConsumeEvents(u16),
    /// 0. `[]` market
    /// 1. `[writable]` OpenOrders
    /// 2. `[writable]` the request queue
    /// 3. `[signer]` the OpenOrders owner
    CancelOrder(CancelOrderInstruction),
    /// 0. `[writable]` market
    /// 1. `[writable]` OpenOrders
    /// 2. `[signer]` the OpenOrders owner
    /// 3. `[writable]` coin vault
    /// 4. `[writable]` pc vault
    /// 5. `[writable]` coin wallet
    /// 6. `[writable]` pc wallet
    /// 7. `[]` vault signer
    /// 8. `[]` spl token program
    /// 9. `[writable]` (optional) referrer pc wallet
    SettleFunds,
    /// 0. `[]` market
    /// 1. `[writable]` OpenOrders
    /// 2. `[writable]` the request queue
    /// 3. `[signer]` the OpenOrders owner
    CancelOrderByClientId(u64),
    /// 0. `[writable]` market
    /// 1. `[signer]` disable authority
    DisableMarket,
    /// 0. `[writable]` market
    /// 1. `[writable]` pc vault
    /// 2. `[signer]` fee sweeping authority
    /// 3. `[writable]` fee receivable account
    /// 4. `[]` vault signer
    /// 5. `[]` spl token program
    SweepFees,
    /// 0. `[writable]` the market
    /// 1. `[writable]` the OpenOrders account to use
    /// 2. `[writable]` the request queue
    /// 3. `[writable]` the (coin or price currency) account paying for the order
    /// 4. `[signer]` owner of the OpenOrders account
    /// 5. `[writable]` coin vault
    /// 6. `[writable]` pc vault
    /// 7. `[]` spl token program
    /// 8. `[]` the rent sysvar
    /// 9. `[]` (optional) the (M)SRM account used for fee discounts
    NewOrderV2(NewOrderInstructionV2),
    /// 0. `[writable]` the market
    /// 1. `[writable]` the OpenOrders account to use
    /// 2. `[writable]` the request queue
    /// 3. `[writable]` the event queue
    /// 4. `[writable]` bids
    /// 5. `[writable]` asks
    /// 6. `[writable]` the (coin or price currency) account paying for the order
    /// 7. `[signer]` owner of the OpenOrders account
    /// 8. `[writable]` coin vault
    /// 9. `[writable]` pc vault
    /// 10. `[]` spl token program
    /// 11. `[]` the rent sysvar
    /// 12. `[]` (optional) the (M)SRM account used for fee discounts
    NewOrderV3(NewOrderInstructionV3),
    /// 0. `[writable]` market
    /// 1. `[writable]` bids
    /// 2. `[writable]` asks
    /// 3. `[writable]` OpenOrders
    /// 4. `[signer]` the OpenOrders owner
    /// 5. `[writable]` event_q
    CancelOrderV2(CancelOrderInstructionV2),
    /// 0. `[writable]` market
    /// 1. `[writable]` bids
    /// 2. `[writable]` asks
    /// 3. `[writable]` OpenOrders
    /// 4. `[signer]` the OpenOrders owner
    /// 5. `[writable]` event_q
    CancelOrderByClientIdV2(u64),
    /// 0. `[writable]` market
    /// 1. `[writable]` the request queue
    /// 2. `[writable]` the event queue
    /// 3. `[writable]` bids
    /// 4. `[writable]` asks
    /// 5. `[writable]` the coin currency wallet account
    /// 6. `[writable]` the price currency wallet account
    /// 7. `[]` signer
    /// 8. `[writable]` coin vault
    /// 9. `[writable]` pc vault
    /// 10. `[]` spl token program
    /// 11. `[]` (optional) the (M)SRM account used for fee discounts
    SendTake(SendTakeInstruction),
    /// 0. `[writable]` OpenOrders
    /// 1. `[signer]` the OpenOrders owner
    /// 2. `[writable]` the destination account to send rent exemption SOL to
    /// 3. `[]` market
    CloseOpenOrders,
    /// 0. `[writable]` OpenOrders
    /// 1. `[signer]` the OpenOrders owner
    /// 2. `[]` market
    /// 3. `[]`
    /// 4. `[signer]` open orders market authority (optional).
    InitOpenOrders,
    /// Removes all orders for a given open orders account from the orderbook.
    ///
    /// 0. `[writable]` market
    /// 1. `[writable]` bids
    /// 2. `[writable]` asks
    /// 3. `[signer]` prune authority
    /// 4. `[]` open orders.
    /// 5. `[]` open orders owner.
    /// 6. `[writable]` event queue.
    Prune(u16),
    /// ... `[writable]` OpenOrders
    /// accounts.len() - 3 `[writable]` market
    /// accounts.len() - 2 `[writable]` event queue
    /// accounts.len() - 1 `[signer]` crank authority
    ConsumeEventsPermissioned(u16),
    /// 0. `[writable]` market
    /// 1. `[writable]` bids
    /// 2. `[writable]` asks
    /// 3. `[writable]` OpenOrders
    /// 4. `[signer]` the OpenOrders owner
    /// 5. `[writable]` event_q
    CancelOrdersByClientIds([u64; 8]),
    /// 0. `[writable]` the market
    /// 1. `[writable]` the OpenOrders account to use
    /// 2. `[writable]` the request queue
    /// 3. `[writable]` the event queue
    /// 4. `[writable]` bids
    /// 5. `[writable]` asks
    /// 6. `[writable]` the (coin or price currency) account paying for the order
    /// 7. `[signer]` owner of the OpenOrders account
    /// 8. `[writable]` coin vault
    /// 9. `[writable]` pc vault
    /// 10. `[]` spl token program
    /// 11. `[]` the rent sysvar
    /// 12. `[]` (optional) the (M)SRM account used for fee discounts
    ReplaceOrderByClientId(NewOrderInstructionV3),
    /// 0. `[writable]` the market
    /// 1. `[writable]` the OpenOrders account to use
    /// 2. `[writable]` the request queue
    /// 3. `[writable]` the event queue
    /// 4. `[writable]` bids
    /// 5. `[writable]` asks
    /// 6. `[writable]` the (coin or price currency) account paying for the order
    /// 7. `[signer]` owner of the OpenOrders account
    /// 8. `[writable]` coin vault
    /// 9. `[writable]` pc vault
    /// 10. `[]` spl token program
    /// 11. `[]` the rent sysvar
    /// 12. `[]` (optional) the (M)SRM account used for fee discounts
    ReplaceOrdersByClientIds(Vec<NewOrderInstructionV3>),
}

impl MarketInstruction {
    pub fn pack(&self) -> Vec<u8> {
        bincode::serialize(&(0u8, self)).unwrap()
    }

    pub fn unpack(versioned_bytes: &[u8]) -> Option<Self> {
        if versioned_bytes.len() < 5 || versioned_bytes.len() > 5 + 8 + 54 * 8
        {
            return None;
        }
        let (&[version], &discrim, data) =
            array_refs![versioned_bytes, 1, 4; ..;];
        if version != 0 {
            return None;
        }
        let discrim = u32::from_le_bytes(discrim);
        Some(match (discrim, data.len()) {
            (0, 34) => MarketInstruction::InitializeMarket({
                let data_array = array_ref![data, 0, 34];
                let fields = array_refs![data_array, 8, 8, 2, 8, 8];
                InitializeMarketInstruction {
                    coin_lot_size: u64::from_le_bytes(*fields.0),
                    pc_lot_size: u64::from_le_bytes(*fields.1),
                    fee_rate_bps: u16::from_le_bytes(*fields.2),
                    vault_signer_nonce: u64::from_le_bytes(*fields.3),
                    pc_dust_threshold: u64::from_le_bytes(*fields.4),
                }
            }),
            (1, 32) => MarketInstruction::NewOrder({
                let data_arr = array_ref![data, 0, 32];
                NewOrderInstructionV1::unpack(data_arr)?
            }),
            (2, 2) => {
                let limit = array_ref![data, 0, 2];
                MarketInstruction::MatchOrders(u16::from_le_bytes(*limit))
            }
            (3, 2) => {
                let limit = array_ref![data, 0, 2];
                MarketInstruction::ConsumeEvents(u16::from_le_bytes(*limit))
            }
            (4, 53) => MarketInstruction::CancelOrder({
                let data_array = array_ref![data, 0, 53];
                let fields = array_refs![data_array, 4, 16, 32, 1];
                let side = match u32::from_le_bytes(*fields.0) {
                    0 => Side::Bid,
                    1 => Side::Ask,
                    _ => return None,
                };
                let order_id = u128::from_le_bytes(*fields.1);
                let owner = cast(*fields.2);
                let &[owner_slot] = fields.3;
                CancelOrderInstruction {
                    side,
                    order_id,
                    owner,
                    owner_slot,
                }
            }),
            (5, 0) => MarketInstruction::SettleFunds,
            (6, 8) => {
                let client_id = array_ref![data, 0, 8];
                MarketInstruction::CancelOrderByClientId(u64::from_le_bytes(
                    *client_id,
                ))
            }
            (7, 0) => MarketInstruction::DisableMarket,
            (8, 0) => MarketInstruction::SweepFees,
            (9, 36) => MarketInstruction::NewOrderV2({
                let data_arr = array_ref![data, 0, 36];
                let (v1_data_arr, v2_data_arr) = array_refs![data_arr, 32, 4];
                let v1_instr = NewOrderInstructionV1::unpack(v1_data_arr)?;
                let self_trade_behavior =
                    SelfTradeBehavior::try_from_primitive(
                        u32::from_le_bytes(*v2_data_arr).try_into().ok()?,
                    )
                    .ok()?;
                v1_instr.add_self_trade_behavior(self_trade_behavior)
            }),
            (10, len) if len == 46 || len == 54 => {
                MarketInstruction::NewOrderV3({
                    let extended_data = match len {
                        46 => Some([data, &i64::MAX.to_le_bytes()].concat()),
                        54 => Some(data.to_vec()),
                        _ => None,
                    }?;
                    let data_arr = array_ref![extended_data, 0, 54];
                    NewOrderInstructionV3::unpack(data_arr)?
                })
            }
            (11, 20) => MarketInstruction::CancelOrderV2({
                let data_arr = array_ref![data, 0, 20];
                CancelOrderInstructionV2::unpack(data_arr)?
            }),
            (12, 8) => {
                let client_id = array_ref![data, 0, 8];
                MarketInstruction::CancelOrderByClientIdV2(u64::from_le_bytes(
                    *client_id,
                ))
            }
            (13, 46) => MarketInstruction::SendTake({
                let data_arr = array_ref![data, 0, 46];
                SendTakeInstruction::unpack(data_arr)?
            }),
            (14, 0) => MarketInstruction::CloseOpenOrders,
            (15, 0) => MarketInstruction::InitOpenOrders,
            (16, 2) => {
                let limit = array_ref![data, 0, 2];
                MarketInstruction::Prune(u16::from_le_bytes(*limit))
            }
            (17, 2) => {
                let limit = array_ref![data, 0, 2];
                MarketInstruction::ConsumeEventsPermissioned(
                    u16::from_le_bytes(*limit),
                )
            }
            // At most 8 client ids, each of which is 8 bytes
            (18, len) if len % 8 == 0 && len <= 8 * 8 => {
                let mut client_ids = [0; 8];
                // convert chunks of 8 bytes to client ids
                for (chunk, client_id) in
                    data.chunks_exact(8).zip(client_ids.iter_mut())
                {
                    *client_id = u64::from_le_bytes(chunk.try_into().unwrap());
                }
                MarketInstruction::CancelOrdersByClientIds(client_ids)
            }
            (19, 54) => MarketInstruction::ReplaceOrderByClientId({
                let data_arr = array_ref![data, 0, 54];
                NewOrderInstructionV3::unpack(data_arr)?
            }),
            (20, len) if len % 54 == 8 && len <= 8 + 8 * 54 => {
                if u64::from_le_bytes(data[0..8].try_into().unwrap())
                    != (data.len() as u64 - 8) / 54
                {
                    return None;
                }

                let new_orders = data[8..]
                    .chunks_exact(54)
                    .map(|chunk| {
                        let chunk_arr = array_ref![chunk, 0, 54];
                        NewOrderInstructionV3::unpack(chunk_arr)
                    })
                    .collect::<Option<Vec<_>>>()?;
                MarketInstruction::ReplaceOrdersByClientIds(new_orders)
            }
            _ => return None,
        })
    }

    #[cfg(test)]
    #[inline]
    pub fn unpack_serde(data: &[u8]) -> Result<Self, ()> {
        match data.split_first() {
            None => Err(()),
            Some((&0u8, rest)) => bincode::deserialize(rest).map_err(|_| ()),
            Some((_, _rest)) => Err(()),
        }
    }
}

pub fn initialize_market(
    market: &Pubkey,
    program_id: &Pubkey,
    coin_mint_pk: &Pubkey,
    pc_mint_pk: &Pubkey,
    coin_vault_pk: &Pubkey,
    pc_vault_pk: &Pubkey,
    authority_pk: Option<&Pubkey>,
    prune_authority_pk: Option<&Pubkey>,
    consume_events_authority_pk: Option<&Pubkey>,
    // srm_vault_pk: &Pubkey,
    bids_pk: &Pubkey,
    asks_pk: &Pubkey,
    req_q_pk: &Pubkey,
    event_q_pk: &Pubkey,
    coin_lot_size: u64,
    pc_lot_size: u64,
    vault_signer_nonce: u64,
    pc_dust_threshold: u64,
) -> Result<solana_sdk::instruction::Instruction, DexError> {
    let data =
        MarketInstruction::InitializeMarket(InitializeMarketInstruction {
            coin_lot_size,
            pc_lot_size,
            fee_rate_bps: 0,
            vault_signer_nonce,
            pc_dust_threshold,
        })
        .pack();

    let market_account = AccountMeta::new(*market, false);

    let bids = AccountMeta::new(*bids_pk, false);
    let asks = AccountMeta::new(*asks_pk, false);
    let req_q = AccountMeta::new(*req_q_pk, false);
    let event_q = AccountMeta::new(*event_q_pk, false);

    let coin_vault = AccountMeta::new(*coin_vault_pk, false);
    let pc_vault = AccountMeta::new(*pc_vault_pk, false);

    let coin_mint = AccountMeta::new_readonly(*coin_mint_pk, false);
    let pc_mint = AccountMeta::new_readonly(*pc_mint_pk, false);

    let rent_sysvar = AccountMeta::new_readonly(RENT_PROGRAM, false);

    let mut accounts = vec![
        market_account,
        req_q,
        event_q,
        bids,
        asks,
        coin_vault,
        pc_vault,
        //srm_vault,
        coin_mint,
        pc_mint,
        //srm_mint,
        rent_sysvar,
    ];
    if let Some(auth) = authority_pk {
        let authority = AccountMeta::new_readonly(*auth, false);
        accounts.push(authority);
        if let Some(prune_auth) = prune_authority_pk {
            let authority = AccountMeta::new_readonly(*prune_auth, false);
            accounts.push(authority);
            if let Some(consume_events_auth) = consume_events_authority_pk {
                let authority =
                    AccountMeta::new_readonly(*consume_events_auth, false);
                accounts.push(authority);
            }
        }
    }

    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn new_order(
    market: &Pubkey,
    open_orders_account: &Pubkey,
    request_queue: &Pubkey,
    event_queue: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    order_payer: &Pubkey,
    open_orders_account_owner: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    spl_token_program_id: &Pubkey,
    rent_sysvar_id: &Pubkey,
    srm_account_referral: Option<&Pubkey>,
    program_id: &Pubkey,
    side: Side,
    limit_price: NonZeroU64,
    max_coin_qty: NonZeroU64,
    order_type: OrderType,
    client_order_id: u64,
    self_trade_behavior: SelfTradeBehavior,
    limit: u16,
    max_native_pc_qty_including_fees: NonZeroU64,
    max_ts: i64,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::NewOrderV3(NewOrderInstructionV3 {
        side,
        limit_price,
        max_coin_qty,
        order_type,
        client_order_id,
        self_trade_behavior,
        limit,
        max_native_pc_qty_including_fees,
        max_ts,
    })
    .pack();
    let mut accounts = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*open_orders_account, false),
        AccountMeta::new(*request_queue, false),
        AccountMeta::new(*event_queue, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*order_payer, false),
        AccountMeta::new_readonly(*open_orders_account_owner, true),
        AccountMeta::new(*coin_vault, false),
        AccountMeta::new(*pc_vault, false),
        AccountMeta::new_readonly(*spl_token_program_id, false),
        AccountMeta::new_readonly(*rent_sysvar_id, false),
    ];
    if let Some(key) = srm_account_referral {
        accounts.push(AccountMeta::new_readonly(*key, false))
    }
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn match_orders(
    program_id: &Pubkey,
    market: &Pubkey,
    request_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    event_queue: &Pubkey,
    coin_fee_receivable_account: &Pubkey,
    pc_fee_receivable_account: &Pubkey,
    limit: u16,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::MatchOrders(limit).pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*request_queue, false),
        AccountMeta::new(*event_queue, false),
        AccountMeta::new(*bids, false),
        AccountMeta::new(*asks, false),
        AccountMeta::new(*coin_fee_receivable_account, false),
        AccountMeta::new(*pc_fee_receivable_account, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn consume_events(
    program_id: &Pubkey,
    open_orders_accounts: Vec<&Pubkey>,
    market: &Pubkey,
    event_queue: &Pubkey,
    coin_fee_receivable_account: &Pubkey,
    pc_fee_receivable_account: &Pubkey,
    limit: u16,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::ConsumeEvents(limit).pack();
    let mut accounts: Vec<AccountMeta> = open_orders_accounts
        .iter()
        .map(|key| AccountMeta::new(**key, false))
        .collect();
    accounts.append(&mut vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*event_queue, false),
        AccountMeta::new(*coin_fee_receivable_account, false),
        AccountMeta::new(*pc_fee_receivable_account, false),
    ]);
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn consume_events_permissioned(
    program_id: &Pubkey,
    open_orders_accounts: Vec<&Pubkey>,
    market: &Pubkey,
    event_queue: &Pubkey,
    consume_events_authority: &Pubkey,
    limit: u16,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::ConsumeEventsPermissioned(limit).pack();
    let mut accounts: Vec<AccountMeta> = open_orders_accounts
        .iter()
        .map(|key| AccountMeta::new(**key, false))
        .collect();
    accounts.append(&mut vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*event_queue, false),
        AccountMeta::new_readonly(*consume_events_authority, true),
    ]);
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn cancel_order(
    program_id: &Pubkey,
    market: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    open_orders_account: &Pubkey,
    open_orders_account_owner: &Pubkey,
    event_queue: &Pubkey,
    side: Side,
    order_id: u128,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::CancelOrderV2(CancelOrderInstructionV2 {
        side,
        order_id,
    })
    .pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*open_orders_account, false),
        AccountMeta::new_readonly(*open_orders_account_owner, true),
        AccountMeta::new(*event_queue, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn settle_funds(
    program_id: &Pubkey,
    market: &Pubkey,
    spl_token_program_id: &Pubkey,
    open_orders_account: &Pubkey,
    open_orders_account_owner: &Pubkey,
    coin_vault: &Pubkey,
    coin_wallet: &Pubkey,
    pc_vault: &Pubkey,
    pc_wallet: &Pubkey,
    referrer_pc_wallet: Option<&Pubkey>,
    vault_signer: &Pubkey,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::SettleFunds.pack();
    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*open_orders_account, false),
        AccountMeta::new_readonly(*open_orders_account_owner, true),
        AccountMeta::new(*coin_vault, false),
        AccountMeta::new(*pc_vault, false),
        AccountMeta::new(*coin_wallet, false),
        AccountMeta::new(*pc_wallet, false),
        AccountMeta::new_readonly(*vault_signer, false),
        AccountMeta::new_readonly(*spl_token_program_id, false),
    ];
    if let Some(key) = referrer_pc_wallet {
        accounts.push(AccountMeta::new(*key, false))
    }
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn cancel_order_by_client_order_id(
    program_id: &Pubkey,
    market: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    open_orders_account: &Pubkey,
    open_orders_account_owner: &Pubkey,
    event_queue: &Pubkey,
    client_order_id: u64,
) -> Result<Instruction, DexError> {
    let data =
        MarketInstruction::CancelOrderByClientIdV2(client_order_id).pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*open_orders_account, false),
        AccountMeta::new_readonly(*open_orders_account_owner, true),
        AccountMeta::new(*event_queue, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn cancel_orders_by_client_order_ids(
    program_id: &Pubkey,
    market: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    open_orders_account: &Pubkey,
    open_orders_account_owner: &Pubkey,
    event_queue: &Pubkey,
    client_order_ids: [u64; 8],
) -> Result<Instruction, DexError> {
    let data =
        MarketInstruction::CancelOrdersByClientIds(client_order_ids).pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*open_orders_account, false),
        AccountMeta::new_readonly(*open_orders_account_owner, true),
        AccountMeta::new(*event_queue, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn disable_market(
    program_id: &Pubkey,
    market: &Pubkey,
    disable_authority_key: &Pubkey,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::DisableMarket.pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*disable_authority_key, true),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn sweep_fees(
    program_id: &Pubkey,
    market: &Pubkey,
    pc_vault: &Pubkey,
    fee_sweeping_authority: &Pubkey,
    fee_receivable_account: &Pubkey,
    vault_signer: &Pubkey,
    spl_token_program_id: &Pubkey,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::SweepFees.pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*pc_vault, false),
        AccountMeta::new_readonly(*fee_sweeping_authority, true),
        AccountMeta::new(*fee_receivable_account, false),
        AccountMeta::new_readonly(*vault_signer, false),
        AccountMeta::new_readonly(*spl_token_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn close_open_orders(
    program_id: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    destination: &Pubkey,
    market: &Pubkey,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::CloseOpenOrders.pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*open_orders, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*destination, false),
        AccountMeta::new_readonly(*market, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn init_open_orders(
    program_id: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    market: &Pubkey,
    market_authority: Option<&Pubkey>,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::InitOpenOrders.pack();
    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*open_orders, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(rent::ID, false),
    ];
    if let Some(market_authority) = market_authority {
        accounts.push(AccountMeta::new_readonly(*market_authority, true));
    }
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn prune(
    program_id: &Pubkey,
    market: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    prune_authority: &Pubkey,
    open_orders: &Pubkey,
    open_orders_owner: &Pubkey,
    event_q: &Pubkey,
    limit: u16,
) -> Result<Instruction, DexError> {
    let data = MarketInstruction::Prune(limit).pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*bids, false),
        AccountMeta::new(*asks, false),
        AccountMeta::new_readonly(*prune_authority, true),
        AccountMeta::new(*open_orders, false),
        AccountMeta::new_readonly(*open_orders_owner, false),
        AccountMeta::new(*event_q, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}
