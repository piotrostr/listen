use std::error::Error;
use std::str::FromStr;
use log::info;

use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};
use solana_sdk::system_instruction::transfer;

use crate::raydium::make_compute_budget_ixs;
use crate::util::env;

// bloxroute.sol
pub const BLOXROUTE_ADDRESS: &str = "HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY";
// PumpFun Global address (not sure what it is, but it is constant among buy ix)
pub const PUMP_GLOBAL_ADDRESS: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FEE_ADDRESS: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";

pub async fn buy_pump_token(mint: Pubkey) -> Result<(), Box<dyn Error>> {
    info!("Buying pump token {}", mint.to_string());
    let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
    // 0.003 sol
    let tip = 3000000;
    let owner = wallet.pubkey();

    let mut ixs = vec![];
    ixs.append(&mut make_compute_budget_ixs(262500, 100000));
    ixs.push(transfer(
        &owner,
        &Pubkey::from_str(BLOXROUTE_ADDRESS)?,
        tip,
    ));
    let _ /* ata */ = &spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let mut ata_ixs = raydium_library::common::create_ata_token_or_not(&owner, &mint, &owner);

    ixs.append(&mut ata_ixs);

    Ok(())
}


/// Interact With Pump.Fun 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
/// Input Accounts
/// #1 - Global: 4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf
/// #2 - Fee Recipient: Pump.fun Fee Account [Writable]
/// #3 - Mint 
/// #4 - Bonding Curve [Writable]
/// #5 - Associated Bonding Curve [Writable]
/// #6 - Associated User [Writable]
/// #7 - User - owner, sender [Writable, Signer, Fee Payer]
/// #8 - System Program (11111111111111111111111111111111)
/// #9 - Token Program (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
/// #10 - Rent (SysvarRent111111111111111111111111111111111)
/// #11 - Event Authority: Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1
/// #12 - Program: Pump.fun Program 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
pub fn pump_swap_ixs(mint: Pubkey) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let _ = [
        AccountMeta::new_readonly(Pubkey::from_str(PUMP_GLOBAL_ADDRESS)?, false),
        AccountMeta::new(Pubkey::from_str(PUMP_FEE_ADDRESS)?, false), // writable
        AccountMeta::new_readonly(mint, false),
    ];
    Ok(vec![])
}
