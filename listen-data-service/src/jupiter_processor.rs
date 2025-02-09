use carbon_core::{
    account::AccountProcessorInputType, error::CarbonResult, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_jupiter_swap_decoder::accounts::JupiterSwapAccount;
use std::sync::Arc;
use tracing::info;

use crate::kv_store::RedisKVStore;
use crate::util::make_kv_store;

#[allow(dead_code)]
pub struct JupiterProcessor {
    kv_store: Arc<RedisKVStore>,
}

impl JupiterProcessor {
    pub fn new() -> Self {
        Self {
            kv_store: make_kv_store().expect("Failed to create KV store"),
        }
    }
}

#[async_trait::async_trait]
impl Processor for JupiterProcessor {
    type InputType = AccountProcessorInputType<JupiterSwapAccount>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (_meta, account) = data;

        println!("account: {:?}", _meta.pubkey);

        match &account.data {
            JupiterSwapAccount::TokenLedger(ledger) => {
                let token_mint = ledger.token_account.to_string();
                let amount = ledger.amount;

                info!("Token: {}, Amount: {}", token_mint, amount);
            }
        }

        Ok(())
    }
}
