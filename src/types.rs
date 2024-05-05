use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    pub data: HashMap<String, PriceData>,
    pub time_taken: f64,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub id: String,
    pub mint_symbol: String,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: Option<u32>,
    pub swap_mode: Option<String>,
    pub dexes: Option<Vec<String>>,
    pub exclude_dexes: Option<Vec<String>>,
    pub restrict_intermediate_tokens: Option<bool>,
    pub only_direct_routes: Option<bool>,
    pub as_legacy_transaction: Option<bool>,
    pub platform_fee_bps: Option<u32>,
    pub max_accounts: Option<u32>,
    pub auto_slippage: Option<bool>,
    pub max_auto_slippage_bps: Option<u32>,
    pub auto_slippage_collision_usd_value: Option<u32>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: Option<String>,
    pub swap_mode: String,
    pub slippage_bps: u64,
    pub platform_fee: Option<PlatformFee>,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub swap_info: Option<SwapInfo>,
    pub context_slot: Option<u64>,
    pub time_taken: Option<f64>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u64,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Serialize)]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub use_shared_accounts: Option<bool>,
    pub fee_account: Option<String>,
    pub tracking_account: Option<String>,
    pub compute_unit_price_micro_lamports: Option<u32>,
    pub prioritization_fee_lamports: Option<u32>,
    pub as_legacy_transaction: Option<bool>,
    pub use_token_ledger: Option<bool>,
    pub destination_token_account: Option<String>,
    pub dynamic_compute_unit_limit: Option<bool>,
    pub skip_user_accounts_rpc_calls: Option<bool>,
    pub quote_response: QuoteResponse,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: Option<u32>,
    pub prioritization_fee_lamports: Option<u32>,
}
