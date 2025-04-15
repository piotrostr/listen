// use crate::distiller::analyst;
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;

use crate::faster100x::{
    data::get_faster100x_data, format::format_wallet_analysis,
};

pub mod data;
pub mod format;
pub mod risk;
pub mod types;

#[tool(description = "
Analyze the distribution and concentration of wallets for a Solana token

Parameters:
- token_address (string): The address of the token to analyze.

Returns:
- Analysis of wallet concentration
- Connected wallet clusters
- Gini index for distribution
- Possible risk signals like high concentration
")]
pub async fn analyze_holder_distribution(
    token_address: String,
) -> Result<serde_json::Value> {
    let data = match get_faster100x_data(&token_address).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Error getting data from Faster100x: {}", e);
            return Err(anyhow!(
                "No data found for token address: {}",
                token_address
            ));
        }
    };

    format_wallet_analysis(&data)
}

#[cfg(test)]
mod tests {
    use crate::faster100x::risk::{
        calculate_gini_index, compute_holder_risk,
    };

    use super::*;

    #[tokio::test]
    async fn test_get_faster100x_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        // Test a known token address
        let data = get_faster100x_data(
            "HEZ6KcNNUKaWvUCBEe4BtfoeDHEHPkCHY9JaDNqrpump",
        )
        .await
        .expect("API call should succeed");

        // Basic data validation
        assert_eq!(
            data.status, "success",
            "API should return success status"
        );

        // Check risk metrics first (before moving data.data)
        let metrics =
            compute_holder_risk(&data).expect("Should compute risk metrics");
        assert!(metrics.gini_index > 0.0, "Should have valid Gini index");
        assert!(
            metrics.isolated.top70_centralization > 0.0,
            "Should have valid centralization"
        );

        // Check holders data
        let response_data = data.data.expect("Response should contain data");
        assert!(
            !response_data.response.data.is_empty(),
            "Should have holder data"
        );
        assert!(
            !response_data.response.fund_graph_data.nodes.is_empty(),
            "Should have graph nodes"
        );
    }

    #[test]
    fn test_gini_index() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let gini = calculate_gini_index(&values);
        assert!(
            (0.0..=1.0).contains(&gini),
            "Gini index should be between 0 and 1"
        );
    }
}
