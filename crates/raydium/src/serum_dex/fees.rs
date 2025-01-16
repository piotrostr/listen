use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum FeeTier {
    Base,
    _SRM2,
    _SRM3,
    _SRM4,
    _SRM5,
    _SRM6,
    _MSRM,
    Stable,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct U64F64(u128);

impl U64F64 {
    const ONE: Self = U64F64(1 << 64);

    #[inline(always)]
    const fn add(self, other: U64F64) -> U64F64 {
        U64F64(self.0 + other.0)
    }

    #[inline(always)]
    const fn div(self, other: U64F64) -> u128 {
        self.0 / other.0
    }

    #[inline(always)]
    const fn mul_u64(self, other: u64) -> U64F64 {
        U64F64(self.0 * other as u128)
    }

    #[inline(always)]
    const fn floor(self) -> u64 {
        (self.0 >> 64) as u64
    }

    #[inline(always)]
    const fn frac_part(self) -> u64 {
        self.0 as u64
    }

    #[inline(always)]
    const fn from_int(n: u64) -> Self {
        U64F64((n as u128) << 64)
    }
}

#[inline(always)]
const fn fee_tenth_of_bps(tenth_of_bps: u64) -> U64F64 {
    U64F64(((tenth_of_bps as u128) << 64) / 100_000)
}

impl FeeTier {
    fn maker_rate(self) -> U64F64 {
        use FeeTier::*;
        match self {
            Stable => fee_tenth_of_bps(5),
            Base | _ => fee_tenth_of_bps(20),
        }
    }

    #[inline]
    pub fn maker_rebate(self, pc_qty: u64) -> u64 {
        let rate = self.maker_rate();
        rate.mul_u64(pc_qty).floor()
    }

    fn taker_rate(self) -> U64F64 {
        self.maker_rate().mul_u64(2)
    }

    #[inline]
    pub fn taker_fee(self, pc_qty: u64) -> u64 {
        let rate = self.taker_rate();
        let exact_fee = rate.mul_u64(pc_qty);
        exact_fee.floor() + (exact_fee.frac_part() != 0) as u64
    }

    #[inline]
    pub fn remove_taker_fee(self, pc_qty_incl_fee: u64) -> u64 {
        let rate = self.taker_rate();
        U64F64::from_int(pc_qty_incl_fee)
            .div(U64F64::ONE.add(rate))
            .try_into()
            .unwrap()
    }
}

#[inline]
pub fn referrer_rebate(amount: u64) -> u64 {
    amount / 2
}
