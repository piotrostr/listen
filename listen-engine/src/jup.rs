use std::str::FromStr;

use anyhow::{anyhow, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub fee_bps: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicSlippage {
    #[serde(rename = "minBps")]
    pub min_bps: i32,
    #[serde(rename = "maxBps")]
    pub max_bps: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: i32,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub context_slot: u64,
    #[serde(rename = "timeTaken")]
    pub time_taken: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: Option<String>,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

#[derive(Serialize)]
pub struct SwapRequest {
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
    #[serde(rename = "useSharedAccounts")]
    pub use_shared_accounts: bool,
    #[serde(rename = "feeAccount")]
    pub fee_account: Option<String>,
    #[serde(rename = "trackingAccount")]
    pub tracking_account: Option<String>,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: Option<u64>,
    #[serde(rename = "asLegacyTransaction")]
    pub as_legacy_transaction: bool,
    #[serde(rename = "useTokenLedger")]
    pub use_token_ledger: bool,
    #[serde(rename = "destinationTokenAccount")]
    pub destination_token_account: Option<String>,
    #[serde(rename = "dynamicComputeUnitLimit")]
    pub dynamic_compute_unit_limit: bool,
    #[serde(rename = "skipUserAccountsRpcCalls")]
    pub skip_user_accounts_rpc_calls: bool,
    #[serde(rename = "dynamicSlippage")]
    pub dynamic_slippage: Option<DynamicSlippage>,
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponse,
}

#[derive(Deserialize, Debug)]
pub struct SwapInstructionsResponse {
    #[serde(rename = "tokenLedgerInstruction")]
    pub token_ledger_instruction: Option<InstructionData>,
    #[serde(rename = "computeBudgetInstructions")]
    pub compute_budget_instructions: Option<Vec<InstructionData>>,
    #[serde(rename = "setupInstructions")]
    pub setup_instructions: Vec<InstructionData>,
    #[serde(rename = "swapInstruction")]
    pub swap_instruction: InstructionData,
    #[serde(rename = "cleanupInstruction")]
    pub cleanup_instruction: Option<InstructionData>,
    #[serde(rename = "addressLookupTableAddresses")]
    pub address_lookup_table_addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
    // other fields not relevant atm
}

#[derive(Deserialize, Debug)]
pub struct InstructionData {
    #[serde(rename = "programId")]
    pub program_id: String,
    pub accounts: Vec<AccountMeta>,
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct AccountMeta {
    pub pubkey: String,
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
    #[serde(rename = "isWritable")]
    pub is_writable: bool,
}

pub struct Jupiter;

impl Jupiter {
    pub async fn fetch_quote(
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<QuoteResponse> {
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
            input_mint, output_mint, amount,
        );

        let response = reqwest::get(&url).await?.json::<QuoteResponse>().await?;
        Ok(response)
    }

    pub async fn swap(
        quote_response: QuoteResponse,
        owner: &Pubkey,
    ) -> Result<VersionedTransaction> {
        let swap_request = serde_json::json!({
            "userPublicKey": owner.to_string(),
            "quoteResponse": quote_response,
            "dynamicSlippage": true
        });
        let client = reqwest::Client::new();
        let raw_res = client
            .post("https://quote-api.jup.ag/v6/swap")
            .json(&swap_request)
            .send()
            .await?;
        if !raw_res.status().is_success() {
            let error = raw_res.text().await.map_err(|e| anyhow!(e))?;
            return Err(anyhow!(error));
        }
        let response = raw_res
            .json::<SwapResponse>()
            .await
            .map_err(|e| anyhow!(e))?;

        let tx: VersionedTransaction = bincode::deserialize(
            &BASE64_STANDARD
                .decode(response.swap_transaction)
                .map_err(|e| anyhow!(e))?,
        )
        .map_err(|e| anyhow!(e))?;

        Ok(tx)
    }

    fn _convert_instruction_data(
        ix_data: InstructionData,
    ) -> Result<solana_sdk::instruction::Instruction> {
        let program_id = Pubkey::from_str(&ix_data.program_id)?;

        let accounts = ix_data
            .accounts
            .into_iter()
            .map(|acc| {
                Ok(solana_sdk::instruction::AccountMeta {
                    pubkey: Pubkey::from_str(&acc.pubkey)?,
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let data = BASE64_STANDARD.decode(ix_data.data)?;

        Ok(solana_sdk::instruction::Instruction {
            program_id,
            accounts,
            data,
        })
    }
}
