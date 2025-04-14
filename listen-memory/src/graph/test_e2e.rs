use super::{Filters, GraphMemory};

#[tokio::test]
async fn test_e2e() {
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory
        .search("what is ryzome?", Filters {}, Some(10))
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

#[tokio::test]
async fn test_get_all_entities() {
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory.client.get_all_entities().await.unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
