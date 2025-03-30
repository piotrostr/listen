use crate::faster100x::risk::{compute_holder_risk, determine_risk_level};
use crate::faster100x::types::Faster100xData;
use anyhow::Result;

pub fn format_wallet_analysis(
    faster_data: &Faster100xData,
) -> Result<serde_json::Value> {
    let risk_metrics = compute_holder_risk(faster_data);

    let max_holder = faster_data.data.as_ref().and_then(|d| {
        d.response.data.iter().max_by(|a, b| {
            a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()
        })
    });
    let token_address = faster_data
        .data
        .as_ref()
        .and_then(|d| d.token_address.clone());
    let updated_at =
        faster_data.data.as_ref().and_then(|d| d.updated_at.clone());

    let analysis = if let Some(metrics) = risk_metrics {
        serde_json::json!({
            "status": "success",
            "token_address": token_address,
            "updated_at": updated_at,
            "max_holder": max_holder.map(|h| {
                serde_json::json!({
                    "address": h.address,
                    "percentage": h.get_percentage() * 100.0,
                })
            }),
            "holder_risk": {
                "isolated_wallets": {
                    "count": metrics.isolated.num_wallets,
                    "total_percentage": metrics.isolated.total_percentage,
                },
                "linked_wallets": {
                    "clusters": metrics.linked.num_clusters,
                    "total_percentage": metrics.linked.total_percentage,
                    "largest_clusters": metrics.linked.clusters.iter()
                        .take(4)
                        .map(|c| serde_json::json!({
                            "wallets": c.num_wallets,
                            "percentage": c.total_percentage,
                        }))
                        .collect::<Vec<_>>(),
                },
                "distribution": {
                    "gini_index": metrics.gini_index,
                    "top70_centralization": metrics.isolated.top70_centralization,
                },
                "risk_level": determine_risk_level(&metrics),
            }
        })
    } else {
        serde_json::json!({
            "status": "error",
            "message": "Failed to compute risk metrics",
            "token_address": token_address,
        })
    };

    Ok(analysis)
}
