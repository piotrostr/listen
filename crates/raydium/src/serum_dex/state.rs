use bytemuck::cast;
use solana_sdk::pubkey::Pubkey;

pub trait ToAlignedBytes {
    fn to_aligned_bytes(&self) -> [u64; 4];
}

impl ToAlignedBytes for Pubkey {
    #[inline]
    fn to_aligned_bytes(&self) -> [u64; 4] {
        cast(self.to_bytes())
    }
}
