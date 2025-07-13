#[cfg(feature = "http")]
use {
    listen_kit::http::server::run_server,
    listen_memory::graph::GraphMemory,
    listen_mongo::MongoClient,
    privy::{config::PrivyConfig, Privy},
    std::sync::Arc,
};

#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let privy_client =
        Privy::new(PrivyConfig::from_env().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?);

    // var is WITHOUT, since default should be with adapter
    let with_adapter = std::env::var("WITHOUT_ADAPTER").is_err();
    // var is WITH, default should be with data routes. data routes require
    // clickhouse and indexer running (see listen-data, listen-adapter)
    let with_data_routes = std::env::var("WITH_DATA_ROUTES").is_ok();
    let redis_url = std::env::var("REDIS_URL").ok();

    let mongo = if std::env::var("WITH_MONGO").is_ok() {
        MongoClient::from_env().await.map(Arc::new).ok()
    } else {
        None
    };
    if mongo.is_none() {
        tracing::warn!("Starting without MongoDB, chats will not be saved");
    };

    let global_memory = if std::env::var("WITH_NEO4J").is_ok() {
        GraphMemory::from_env().await.map(Arc::new).ok()
    } else {
        None
    };

    if global_memory.is_none() {
        tracing::warn!("Starting without global memory");
    };

    #[cfg(feature = "engine")]
    {
        tokio::select! {
            res1 = listen_engine::server::run() => res1,
            res2 = run_server(
                privy_client,
                mongo.clone(),
                global_memory.clone(),
                with_adapter,
                with_data_routes,
                redis_url,
            ) => res2,
        }
    }

    #[cfg(not(feature = "engine"))]
    {
        // Run server with just the Privy client
        // Agents will be created dynamically based on requests
        run_server(
            privy_client,
            mongo,
            global_memory,
            with_adapter,
            with_data_routes,
            redis_url,
        )
        .await
    }
}

#[cfg(not(feature = "http"))]
fn main() {
    tracing::warn!("This binary requires the 'http' feature");
}
