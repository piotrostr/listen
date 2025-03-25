use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

#[derive(Debug, Default)]
pub struct SwapMetrics {
    pub total_swaps_processed: AtomicU64,
    pub successful_swaps: AtomicU64,
    pub failed_swaps: AtomicU64,
    pub skipped_tiny_swaps: AtomicU64,
    pub skipped_zero_swaps: AtomicU64,
    pub skipped_unexpected_number_of_tokens: AtomicU64,
    pub skipped_no_metadata: AtomicU64,
    pub skipped_non_wsol: AtomicU64,
    pub message_send_success: AtomicU64,
    pub message_send_failure: AtomicU64,
    pub db_insert_success: AtomicU64,
    pub db_insert_failure: AtomicU64,
    pub multi_hop_swap: AtomicU64,
    pub kv_insert_success: AtomicU64,
    pub kv_insert_failure: AtomicU64,
    pub pending_swaps: AtomicU64,
    pub latest_update_slot: AtomicU64,
    pub raydium_amm_v4_swaps: AtomicU64,
    pub raydium_clmm_swaps: AtomicU64,
    pub raydium_cpmm_swaps: AtomicU64,
    pub meteora_dlmm_swaps: AtomicU64,
    pub whirlpools_swaps: AtomicU64,
    pub pump_swaps: AtomicU64,
}

impl SwapMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn increment_raydium_amm_v4_swaps(&self) {
        self.raydium_amm_v4_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_raydium_clmm_swaps(&self) {
        self.raydium_clmm_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_raydium_cpmm_swaps(&self) {
        self.raydium_cpmm_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_meteora_dlmm_swaps(&self) {
        self.meteora_dlmm_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_whirlpools_swaps(&self) {
        self.whirlpools_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_pump_swaps(&self) {
        self.pump_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_total_swaps(&self) {
        let count = self.total_swaps_processed.fetch_add(1, Ordering::Relaxed);
        // println!("total swaps processed: {}", count);
        if (count + 1) % 5000 == 0 {
            self.log_metrics();
        }
    }

    pub fn increment_successful_swaps(&self) {
        self.successful_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed_swaps(&self) {
        self.failed_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped_tiny_swaps(&self) {
        self.skipped_tiny_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped_zero_swaps(&self) {
        self.skipped_zero_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped_unexpected_number_of_tokens(&self) {
        self.skipped_unexpected_number_of_tokens
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped_no_metadata(&self) {
        self.skipped_no_metadata.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped_non_wsol(&self) {
        self.skipped_non_wsol.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_db_insert_success(&self) {
        self.db_insert_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_db_insert_failure(&self) {
        self.db_insert_failure.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_message_send_success(&self) {
        self.message_send_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_message_send_failure(&self) {
        self.message_send_failure.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_multi_hop_swap(&self) {
        self.multi_hop_swap.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_kv_insert_success(&self) {
        self.kv_insert_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_kv_insert_failure(&self) {
        self.kv_insert_failure.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_pending_swaps(&self) {
        self.pending_swaps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_pending_swaps(&self) {
        self.pending_swaps.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn set_latest_update_slot(&self, slot: u64) {
        self.latest_update_slot.store(slot, Ordering::Relaxed);
    }

    fn log_metrics(&self) {
        let total = self.total_swaps_processed.load(Ordering::Relaxed);
        let raydium_amm_v4 = self.raydium_amm_v4_swaps.load(Ordering::Relaxed);
        let raydium_cpmm = self.raydium_cpmm_swaps.load(Ordering::Relaxed);
        let meteora_dlmm = self.meteora_dlmm_swaps.load(Ordering::Relaxed);
        let raydium_clmm = self.raydium_clmm_swaps.load(Ordering::Relaxed);
        let whirlpools = self.whirlpools_swaps.load(Ordering::Relaxed);
        let pump = self.pump_swaps.load(Ordering::Relaxed);
        let pending = self.pending_swaps.load(Ordering::Relaxed);
        let successful = self.successful_swaps.load(Ordering::Relaxed);
        let failed = self.failed_swaps.load(Ordering::Relaxed);
        let tiny = self.skipped_tiny_swaps.load(Ordering::Relaxed);
        let zero = self.skipped_zero_swaps.load(Ordering::Relaxed);
        let unexpected = self
            .skipped_unexpected_number_of_tokens
            .load(Ordering::Relaxed);
        let non_wsol = self.skipped_non_wsol.load(Ordering::Relaxed);
        let no_metadata = self.skipped_no_metadata.load(Ordering::Relaxed);
        let message_send_success =
            self.message_send_success.load(Ordering::Relaxed);
        let message_send_failure =
            self.message_send_failure.load(Ordering::Relaxed);
        let db_insert_success = self.db_insert_success.load(Ordering::Relaxed);
        let db_insert_failure = self.db_insert_failure.load(Ordering::Relaxed);
        let multi_hop = self.multi_hop_swap.load(Ordering::Relaxed);
        let kv_insert_success = self.kv_insert_success.load(Ordering::Relaxed);
        let kv_insert_failure = self.kv_insert_failure.load(Ordering::Relaxed);

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let latest_update_slot =
            self.latest_update_slot.load(Ordering::Relaxed);

        info!(
            "Swap Processing Metrics:\n\
             Total Processed: {}\n\
             Raydium AMM V4: {}\n\
             Raydium CPMM: {}\n\
             Raydium CLMM: {}\n\
             Meteora DLMM: {}\n\
             Whirlpools: {}\n\
             PumpSwap: {}\n\
             Pending: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {}\n\
             Skipped (tiny): {}\n\
             Skipped (zero): {}\n\
             Skipped (unexpected tokens): {}\n\
             Skipped (non-wSOL): {}\n\
             Skipped (no metadata): {}\n\
             Message Send Success: {}\n\
             Message Send Failure: {}\n\
             DB Insert Success: {}\n\
             DB Insert Failure: {}\n\
             Multi-hop Swaps: {}\n\
             KV Insert Success: {}\n\
             KV Insert Failure: {}\n\
             Latest Update Slot: {}",
            total,
            raydium_amm_v4,
            raydium_cpmm,
            raydium_clmm,
            meteora_dlmm,
            whirlpools,
            pump,
            pending,
            successful,
            success_rate,
            failed,
            tiny,
            zero,
            unexpected,
            non_wsol,
            no_metadata,
            message_send_success,
            message_send_failure,
            db_insert_success,
            db_insert_failure,
            multi_hop,
            kv_insert_success,
            kv_insert_failure,
            latest_update_slot,
        );
    }
}
