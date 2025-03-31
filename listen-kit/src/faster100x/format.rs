use crate::faster100x::risk::{compute_holder_risk, determine_risk_level};
use crate::faster100x::types::Faster100xData;
use anyhow::Result;

pub fn format_wallet_analysis(
    faster_data: &Faster100xData,
) -> Result<serde_json::Value> {
    let token_address = faster_data
        .data
        .as_ref()
        .and_then(|d| d.token_address.clone());

    let updated_at =
        faster_data.data.as_ref().and_then(|d| d.updated_at.clone());

    let max_holder = faster_data.data.as_ref().and_then(|d| {
        d.response.data.iter().max_by(|a, b| {
            a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()
        })
    });

    let mut risk_metrics = match compute_holder_risk(faster_data) {
        Some(metrics) => metrics,
        None => {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "Failed to compute risk metrics",
                "token_address": token_address,
            }));
        }
    };

    risk_metrics.linked.clusters.sort_by(|a, b| {
        b.total_percentage.partial_cmp(&a.total_percentage).unwrap()
    });

    Ok(serde_json::json!({
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
                "count": risk_metrics.isolated.num_wallets,
                "total_percentage": risk_metrics.isolated.total_percentage,
            },
            "linked_wallets": {
                "clusters": risk_metrics.linked.num_clusters,
                "total_percentage": risk_metrics.linked.total_percentage,
                "largest_clusters": risk_metrics.linked.clusters
                    .iter()
                    .take(5)
                    .map(|c| serde_json::json!({
                        "wallets": c.num_wallets,
                        "percentage": c.total_percentage,
                    }))
                    .collect::<Vec<_>>(),
            },
            "distribution": {
                "gini_index": risk_metrics.gini_index,
                "top70_centralization": risk_metrics.isolated.top70_centralization,
            },
            "risk_level": determine_risk_level(&risk_metrics),
        }
    }))
}
