use anyhow::Result;
use bb8_redis::redis::{cmd, pipe};
use listen_engine::redis::client::{RedisClient, RedisClientError};
use listen_engine::{engine::error::EngineError, redis::client::make_redis_client};
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tracing::info;
use uuid::Uuid;

async fn create_pipeline_via_api(
    client: &Client,
    _symbol: &str,
    _price_threshold: f64,
    semaphore: Arc<Semaphore>,
) -> Result<()> {
    // Acquire semaphore permit
    let _permit = semaphore.acquire().await.expect("Semaphore closed");

    let _step_id = Uuid::new_v4();

    // Create pipeline request using the server's expected format
    let request = serde_json::json!({
        // TODO this script has fulfilled its purpose, in the future it would be required to
        // intereceipt privy auth header or loophole a way to insert internally
    });
    let response = client
        .post("http://localhost:6966/api/pipeline")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        info!(
            "Response: {:?}",
            response.json::<serde_json::Value>().await?
        );
        return Err(anyhow::anyhow!("Failed to create pipeline"));
    }

    Ok(())
}

async fn cleanup_test_pipelines(redis_client: &RedisClient) -> Result<(), EngineError> {
    let mut conn = redis_client
        .get_connection()
        .await
        .map_err(EngineError::RedisClientError)?;

    // Get all pipeline keys
    let pipeline_keys: Vec<String> = cmd("KEYS")
        .arg("pipeline:*")
        .query_async(&mut *conn)
        .await
        .map_err(|e| EngineError::RedisClientError(RedisClientError::RedisError(e)))?;

    // Delete all pipelines in batches
    let mut pipe = pipe();
    for key in pipeline_keys {
        pipe.del(key);
    }

    let _: () = pipe
        .query_async(&mut *conn)
        .await
        .map_err(|e| EngineError::RedisClientError(RedisClientError::RedisError(e)))?;

    Ok(())
}

async fn get_top_volume_tokens() -> Result<HashSet<String>> {
    let client = Client::new();
    let response = client
        .get("https://api.listen-rs.com/top-tokens")
        .send()
        .await?;

    if !response.status().is_success() {
        println!("Response: {:?}", response.text().await?);
        return Err(anyhow::anyhow!("Failed to query ClickHouse"));
    }

    let data: serde_json::Value = response.json().await?;

    // Extract token names from the response
    let tokens: HashSet<String> = data
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
        .iter()
        .filter_map(|row| row["name"].as_str().map(String::from))
        .collect();

    println!("Grabbing {} tokens from clickhouse", tokens.len());

    Ok(tokens)
}

#[tokio::test]
async fn test_engine_scalability() -> Result<()> {
    tracing_subscriber::fmt::init();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }
    listen_engine::metrics::init_metrics();

    // Start the server in the background
    let server_handle = tokio::spawn(async {
        listen_engine::server::run()
            .await
            .expect("Server failed to start");
    });

    // Give the server a moment to start up
    sleep(Duration::from_secs(2)).await;

    let redis_client = make_redis_client().await?;

    // Create HTTP client for API requests
    let client = Client::new();

    // Get top volume tokens from ClickHouse
    let symbols = get_top_volume_tokens().await?;

    info!("Found {} active tokens", symbols.len());
    assert!(!symbols.is_empty(), "No active tokens found");

    // Create a semaphore to limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(150)); // Allow max 150 concurrent requests

    // Create pipelines for each symbol via API
    let mut futures = Vec::new();
    for symbol in &symbols {
        for threshold in [100.0, 500.0, 1000.0] {
            futures.push(create_pipeline_via_api(
                &client,
                symbol,
                threshold,
                semaphore.clone(),
            ));
        }
    }

    info!("Launching {} pipeline creation requests...", futures.len());
    let results = futures_util::future::join_all(futures).await;
    let pipeline_count = results.len();
    for result in results {
        result?;
    }

    info!("Created {} pipelines", pipeline_count);

    // Let it run for 30 seconds
    sleep(Duration::from_secs(30)).await;

    // Cleanup
    server_handle.abort();

    // Clean up all test pipelines from Redis
    cleanup_test_pipelines(&redis_client).await?;

    Ok(())
}
