use super::{Filters, GraphMemory};

#[tokio::test]
async fn test_e2e() {
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory
        .search("tell me about arc", Filters {}, Some(10))
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
