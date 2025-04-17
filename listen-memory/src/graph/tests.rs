use super::{Filters, GraphMemory};
use crate::graph::EntityInfo;
use std::collections::HashMap;

#[tokio::test]
async fn test_add() {
    let data = "Paris is the capital of France, France is west of Germany";
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory.add(data, Filters {}).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

#[tokio::test]
async fn test_retrieve_nodes() {
    let data = "Paris is the capital of France, France is west of Germany";
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let map = graph_memory.retrieve_nodes(data, Filters {}).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&map).unwrap());
}

#[tokio::test]
#[ignore]
async fn test_establish_nodes_relations() {
    let data = "The ARC token (token:solana:61V...) is developed by the Arc project (project:arc). You can find them at @arcdotfun (user:twitter:arcdotfun).";

    let graph_memory = GraphMemory::from_env().await.unwrap();

    let mut entity_info_map: HashMap<String, EntityInfo> = HashMap::new();
    entity_info_map.insert(
        "token:solana:61v...".to_string(),
        EntityInfo {
            name: "ARC token".to_string(),
            entity_type: "token".to_string(),
        },
    );
    entity_info_map.insert(
        "project:arc".to_string(),
        EntityInfo {
            name: "Arc project".to_string(),
            entity_type: "project".to_string(),
        },
    );
    entity_info_map.insert(
        "user:twitter:arcdotfun".to_string(),
        EntityInfo {
            name: "@arcdotfun".to_string(),
            entity_type: "user".to_string(),
        },
    );

    let graph_entities = graph_memory
        .establish_nodes_relations(data, Filters {}, entity_info_map)
        .await
        .unwrap();

    println!(
        "Established Relations:\n{}",
        serde_json::to_string_pretty(&graph_entities).unwrap()
    );

    assert!(!graph_entities.is_empty(), "Should establish relations");

    let has_developer_relation = graph_entities
        .iter()
        .any(|rel| rel.source == "token:solana:61v..." && rel.destination == "project:arc");
    let has_handle_relation = graph_entities
        .iter()
        .any(|rel| rel.source == "project:arc" && rel.destination == "user:twitter:arcdotfun");

    println!(
        "Presence check: Token->Project? {}, Project->User? {}",
        has_developer_relation, has_handle_relation
    );
}

#[tokio::test]
async fn test_search() {
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory
        .search(
            "what is the capital of country that shares west border with Germany?",
            Filters {},
            Some(10),
        )
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
