use anyhow::Result;
use bb8_redis::redis::{cmd, pipe};
use listen_engine::redis::client::{RedisClient, RedisClientError};
use listen_engine::server::CreatePipelineRequest;
use listen_engine::{
    engine::{
        pipeline::{Action, Condition, ConditionType, PipelineStep, Status},
        EngineError,
    },
    redis::client::make_redis_client,
};
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

async fn create_pipeline_via_api(
    client: &Client,
    user_id: &str,
    symbol: &str,
    price_threshold: f64,
) -> Result<()> {
    let step_id = Uuid::new_v4();

    // Create pipeline request using the server's expected format
    let request = CreatePipelineRequest {
        user_id: user_id.to_string(),
        current_steps: vec![step_id],
        steps: {
            let mut steps = HashMap::new();
            steps.insert(
                step_id,
                PipelineStep {
                    id: step_id,
                    action: Action::Notification(listen_engine::engine::pipeline::Notification {
                        message: format!("Price alert for {}", symbol),
                    }),
                    conditions: vec![Condition {
                        condition_type: ConditionType::PriceAbove {
                            asset: symbol.to_string(),
                            threshold: price_threshold,
                        },
                        triggered: false,
                        last_evaluated: None,
                    }],
                    next_steps: vec![],
                    status: Status::Pending,
                },
            );
            steps
        },
    };

    let response = client
        .post("http://localhost:6966/api/pipeline")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        println!(
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
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
    let mut conn = redis_client
        .get_connection()
        .await
        .map_err(EngineError::RedisClientError)?;

    // Create HTTP client for API requests
    let client = Client::new();

    // Get all symbol keys
    let symbols: HashSet<String> = cmd("KEYS")
        .arg("solana:metadata:*")
        .query_async(&mut *conn)
        .await
        .map_err(|e| EngineError::RedisClientError(RedisClientError::RedisError(e)))?;

    println!("Found {} symbols", symbols.len());
    assert!(
        symbols.len() >= 100,
        "Need at least 100 symbols for the test"
    );

    // Create pipelines for each symbol via API
    let mut futures = Vec::new();
    for symbol in &symbols {
        for threshold in [100.0, 500.0, 1000.0] {
            futures.push(create_pipeline_via_api(
                &client,
                "test_user",
                symbol,
                threshold,
            ));
        }
    }
    let results = futures_util::future::join_all(futures).await;
    let pipeline_count = results.len();
    for result in results {
        result?;
    }

    println!("Created {} pipelines", pipeline_count);

    // Let it run for 30 seconds
    sleep(Duration::from_secs(30)).await;

    // Cleanup
    server_handle.abort();

    // Clean up all test pipelines from Redis
    cleanup_test_pipelines(&redis_client).await?;

    Ok(())
}
