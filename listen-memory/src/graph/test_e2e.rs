use super::{Filters, GraphMemory};
use neo4rs::query;

#[tokio::test]
async fn test_e2e() {
    println!("Connecting for cleanup...");
    let cleanup_graph = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password")
        .await
        .expect("Failed to connect for cleanup");

    println!("Dropping index if exists...");
    let _drop_result = cleanup_graph
        .run(query("DROP INDEX entityEmbeddingIndex IF EXISTS"))
        .await;
    println!("Index dropped (if existed).");

    println!("Deleting all entities...");
    cleanup_graph
        .run(query("MATCH (n) DETACH DELETE n"))
        .await
        .expect("Failed to delete entities");
    println!("All entities deleted.");

    println!("Running test logic: Initializing GraphMemory...");
    let graph_memory = GraphMemory::from_env().await.unwrap();

    println!("Performing search...");
    let result = graph_memory
        .search("what is ryzome?", Filters {}, Some(10))
        .await
        .expect("Search failed");

    println!(
        "Search Result:\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    assert!(result.is_empty(), "Expected empty results from empty DB");
}

#[tokio::test]
async fn test_get_all_entities() {
    let graph_memory = GraphMemory::from_env().await.unwrap();
    let result = graph_memory.client.get_all_entities().await.unwrap();
    println!(
        "Get All Entities Result:\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );
    assert!(result.is_empty());
}

#[tokio::test]
#[ignore = "this deletes everything"]
async fn test_delete_all_entities() {
    println!("MANUAL CLEANUP: Deleting all entities...");
    let graph = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password")
        .await
        .unwrap();
    graph.run(query("MATCH (n) DETACH DELETE n")).await.unwrap();
    println!("MANUAL CLEANUP: Done deleting.");
}

#[tokio::test]
#[ignore = "this drops the index"]
async fn test_drop_index() {
    println!("MANUAL CLEANUP: Dropping index...");
    let graph = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password")
        .await
        .unwrap();
    let _ = graph
        .run(query("DROP INDEX entityEmbeddingIndex IF EXISTS"))
        .await;
    println!("MANUAL CLEANUP: Done dropping index.");
}

#[tokio::test]
async fn test_insert_samples() {
    tracing_subscriber::fmt::init();
    let graph_memory = GraphMemory::from_env().await.unwrap();
    for fname in std::fs::read_dir(
        std::env::var("HOME").unwrap() + "/solana/listen/listen-kit/tool_output_samples",
    )
    .unwrap()
    {
        let path = fname.unwrap().path();
        let content = std::fs::read_to_string(path.clone()).unwrap();
        if content.is_empty() {
            continue;
        }
        tracing::info!("inserting {}", path.display());
        graph_memory.add(&content, Filters {}).await.unwrap();
    }
}
