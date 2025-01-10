use solana_sdk::{pubkey, pubkey::Pubkey};


pub const SOLANA_PROGRAM_ID: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY: Pubkey = pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

pub const RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY_TESTNET: Pubkey = pubkey!("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8");

pub const RAYDIUM_AUTHORITY_V4_PUBKEY: Pubkey = pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");

pub const USDC_TOKEN_PUBKEY: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub const OPENBOOK_PROGRAM_ID: Pubkey = pubkey!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");

pub const FEE_PROGRAM_ID: Pubkey = pubkey!("7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5");

pub const JITO_TIP_PUBKEY: Pubkey = pubkey!("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY");

pub const RAYDIUM_AMM_PUBKEY: Pubkey = pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"); // TODO: dublicate of RAYDIUM_AUTHORITY_V4_PUBKEY

// TODO
// another rug method is as in case of Fwnf2vDqbHv6GH4eXQHpYmqSMynHrW2yBz8dXxExE5Kq
// initial launch with LP burn, mint/freeze revoked but a large instant buy
// staight aftewards, often from multiple accounts, followed by steady rise
// and a darth maul afterwards
pub fn ruggers() -> Vec<String> {
    vec![
        "3jAhNEb1SgTvgXgsXJrB44jK5opHbxr7NbpPjx9aJE4t".to_string(),
        "32A1b5pbYyqgrtcg49UfpuWmTqz9hrQeE2du91A6pxX8".to_string(),
    ]
}
