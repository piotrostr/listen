use super::{Filters, GraphMemory};
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
async fn test_establish_nodes_relations() {
    let data = "Paris is the capital of France, France is west of Germany";
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let entity_type_map: HashMap<String, String> = serde_json::from_str(
        r#"
            {
                "Paris": "city",
                "France": "country",
                "Germany": "country"
            }
            "#,
    )
    .unwrap();
    let graph_entities = graph_memory
        .establish_nodes_relations(data, Filters {}, entity_type_map)
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&graph_entities).unwrap());
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
