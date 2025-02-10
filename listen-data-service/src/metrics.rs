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
}

impl SwapMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_total_swaps(&self) {
        let count = self.total_swaps_processed.fetch_add(1, Ordering::Relaxed);
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

    fn log_metrics(&self) {
        let total = self.total_swaps_processed.load(Ordering::Relaxed);
        let successful = self.successful_swaps.load(Ordering::Relaxed);
        let failed = self.failed_swaps.load(Ordering::Relaxed);
        let tiny = self.skipped_tiny_swaps.load(Ordering::Relaxed);
        let zero = self.skipped_zero_swaps.load(Ordering::Relaxed);
        let unexpected = self
            .skipped_unexpected_number_of_tokens
            .load(Ordering::Relaxed);

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        info!(
            "Swap Processing Metrics:\n\
             Total Processed: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {}\n\
             Skipped (tiny): {}\n\
             Skipped (zero): {}\n\
             Skipped (unexpected tokens): {}",
            total, successful, success_rate, failed, tiny, zero, unexpected
        );
    }
}
