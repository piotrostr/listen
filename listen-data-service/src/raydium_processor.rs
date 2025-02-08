use carbon_core::{
    account::AccountProcessorInputType, error::CarbonResult, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::accounts::{amm_info::AmmInfo, RaydiumAmmV4Account};
use std::sync::Arc;

pub struct RaydiumAmmV4AccountProcessor;

#[async_trait::async_trait]
impl Processor for RaydiumAmmV4AccountProcessor {
    type InputType = AccountProcessorInputType<RaydiumAmmV4Account>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (_meta, account) = data;
        match &account.data {
            RaydiumAmmV4Account::AmmInfo(pool) => {
                let (coin_price, pc_price) = calculate_pool_prices(pool);
                println!("\nAccount: {:#?}", _meta.pubkey);
                println!("Coin price in PC: {}", coin_price);
                println!("PC price in Coin: {}", pc_price);
            }
            _ => {
                println!("\nUnnecessary Account: {:#?}", _meta.pubkey);
            }
        };

        Ok(())
    }
}

fn calculate_pool_prices(pool: &AmmInfo) -> (f64, f64) {
    // Get swap amounts from output data
    let swap_coin_in = pool.out_put.swap_coin_in_amount as f64;
    let swap_pc_out = pool.out_put.swap_pc_out_amount as f64;
    let swap_pc_in = pool.out_put.swap_pc_in_amount as f64;
    let swap_coin_out = pool.out_put.swap_coin_out_amount as f64;

    // Calculate prices using swap ratios
    // Price of coin in terms of pc (how much pc you get for 1 coin)
    let coin_price_in_pc = if swap_coin_in > 0.0 {
        (swap_pc_out / swap_coin_in)
            * (10f64.powi(pool.pc_decimals as i32) / 10f64.powi(pool.coin_decimals as i32))
    } else {
        0.0
    };

    // Price of pc in terms of coin (how much coin you get for 1 pc)
    let pc_price_in_coin = if swap_pc_in > 0.0 {
        (swap_coin_out / swap_pc_in)
            * (10f64.powi(pool.coin_decimals as i32) / 10f64.powi(pool.pc_decimals as i32))
    } else {
        0.0
    };

    (coin_price_in_pc, pc_price_in_coin)
}
