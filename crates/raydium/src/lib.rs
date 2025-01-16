pub mod amm;
pub mod program;
pub mod serum_dex;
pub mod common;

use std::error::Error;

use log::debug;
use spl_token::state::Mint;

pub fn get_burn_pct(
    mint_data: Mint,
    result: amm::CalculateResult,
) -> Result<f64, Box<dyn Error>> {
    // Calculate divisor for token decimals
    let base = 10u64;
    let divisor = base.pow(mint_data.decimals as u32);

    // Convert lp_reserve and supply to proper scale
    let lp_reserve = result.pool_lp_amount as f64 / divisor as f64;
    let supply = mint_data.supply as f64 / divisor as f64;

    // Calculate max_lp_supply and burn_amount
    let max_lp_supply = lp_reserve.max(supply);
    let burn_amount = max_lp_supply - supply;

    // Avoid division by zero and ensure correct burn percentage calculation
    let burn_pct = if max_lp_supply > 0.0 {
        (burn_amount / max_lp_supply) * 100.0
    } else {
        0.0
    };

    debug!(
        "LP total: {}, LP pooled: {}, LP burnt: {}",
        max_lp_supply, lp_reserve, burn_amount
    );

    Ok(burn_pct)
}
