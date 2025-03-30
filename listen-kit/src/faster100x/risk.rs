use petgraph::graph::NodeIndex;
use petgraph::{Graph, Undirected};

use crate::faster100x::types::{
    Cluster, Faster100xData, HolderRisk, IsolatedHolders, LinkedHolders,
};
use std::collections::HashMap;

pub fn determine_risk_level(metrics: &HolderRisk) -> &'static str {
    let gini = metrics.gini_index;
    let centralization = metrics.isolated.top70_centralization;

    if gini > 80.0 || centralization > 90.0 {
        "Extremely high"
    } else if gini > 70.0 || centralization > 80.0 {
        "Very high"
    } else if gini > 60.0 || centralization > 70.0 {
        "High"
    } else if gini > 50.0 || centralization > 60.0 {
        "Moderate"
    } else if gini > 40.0 || centralization > 50.0 {
        "Low"
    } else {
        "Very low"
    }
}

pub fn compute_holder_risk(
    faster_data: &Faster100xData,
) -> Option<HolderRisk> {
    let response_data = faster_data.data.as_ref()?;
    let fund_graph_data = &response_data.response.fund_graph_data;
    let holdings_list = &response_data.response.data;

    // Map address -> percentage from 'data'
    let mut wallet_holdings: HashMap<String, f64> = HashMap::new();
    for holding in holdings_list {
        wallet_holdings
            .insert(holding.address.clone(), holding.get_percentage());
    }

    // Trova il wallet con la percentuale più alta
    let max_wallet = holdings_list.iter().max_by(|a, b| {
        a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()
    });

    if let Some(max_wallet) = max_wallet {
        tracing::debug!(
            "[Faster100x] Wallet with highest concentration: {} with {}% of supply",
            max_wallet.address,
            max_wallet.get_percentage() * 100.0
        );
    }

    // Extract ALL nodes from fund_graph_data
    let nodes_list = &fund_graph_data.nodes;
    if nodes_list.is_empty() {
        tracing::error!(
            "[Faster100x] No nodes found in fund_graph_data['nodes']"
        );
        return None;
    }

    // Create a graph with petgraph::Graph
    let mut graph = Graph::<String, (), Undirected>::new_undirected();

    // Map to keep track of String -> NodeIndex
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Create graph nodes
    for node_info in nodes_list {
        let addr = node_info.id.clone();
        let idx = graph.add_node(addr.clone());
        node_indices.insert(addr, idx);
    }

    // Add edges
    for link in &fund_graph_data.links {
        if let (Some(&source_idx), Some(&target_idx)) = (
            node_indices.get(&link.source),
            node_indices.get(&link.target),
        ) {
            graph.add_edge(source_idx, target_idx, ());
        }
    }

    // Analyze connected components
    let mut isolated_wallets = Vec::new();
    let mut linked_clusters = Vec::new();

    let _num_components = petgraph::algo::connected_components(&graph);
    let node_to_component: HashMap<NodeIndex, usize> = graph
        .node_indices()
        .map(|node| {
            let component = 0; // Default component

            for (i, component_nodes) in
                petgraph::algo::kosaraju_scc(&graph).iter().enumerate()
            {
                if component_nodes.contains(&node) {
                    return (node, i);
                }
            }

            (node, component)
        })
        .collect();

    // Group nodes by component
    let mut components: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    for (node, component) in node_to_component {
        components.entry(component).or_default().push(node);
    }

    // Process each component
    for (_, nodes) in components {
        if nodes.len() == 1 {
            // Isolated wallet
            let node_idx = nodes[0];
            let addr = graph[node_idx].clone();
            isolated_wallets.push(addr);
        } else {
            // Linked cluster
            let cluster_wallets: Vec<String> = nodes
                .iter()
                .map(|&node_idx| graph[node_idx].clone())
                .collect();

            let total_pct = cluster_wallets
                .iter()
                .filter_map(|addr| wallet_holdings.get(addr))
                .sum::<f64>()
                * 100.0;

            linked_clusters.push(Cluster {
                cluster_wallets,
                num_wallets: nodes.len(),
                total_percentage: total_pct,
            });
        }
    }

    // Calculate centralization on top 70 positions
    let centralization_score = if !wallet_holdings.is_empty() {
        let mut sorted_amounts: Vec<f64> =
            wallet_holdings.values().copied().collect();
        sorted_amounts.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let n = 70.min(sorted_amounts.len());
        sorted_amounts.iter().take(n).sum::<f64>() * 100.0
    } else {
        0.0
    };

    // Calculate total percentages
    let total_isolated_percentage = isolated_wallets
        .iter()
        .filter_map(|addr| wallet_holdings.get(addr))
        .sum::<f64>()
        * 100.0;

    let total_linked_percentage = linked_clusters
        .iter()
        .map(|cluster| cluster.total_percentage)
        .sum();

    // Calculate Gini Index
    let gini_index = calculate_gini_index(
        &wallet_holdings.values().copied().collect::<Vec<f64>>(),
    );

    Some(HolderRisk {
        isolated: IsolatedHolders {
            num_wallets: isolated_wallets.len(),
            total_percentage: total_isolated_percentage,
            top70_centralization: centralization_score,
        },
        linked: LinkedHolders {
            num_clusters: linked_clusters.len(),
            clusters: linked_clusters,
            total_percentage: total_linked_percentage,
        },
        gini_index: gini_index * 100.0,
    })
}

pub fn calculate_gini_index(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let n = sorted_values.len() as f64;
    let sum = sorted_values.iter().sum::<f64>();

    if sum == 0.0 {
        return 0.0;
    }

    let mut index_sum = 0.0;
    for (i, value) in sorted_values.iter().enumerate() {
        index_sum += (2.0 * (i + 1) as f64 - n - 1.0) * value;
    }

    (index_sum / (n * sum)).abs()
}
