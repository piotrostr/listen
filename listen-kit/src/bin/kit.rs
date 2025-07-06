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
            res2 = run_server(privy_client, mongo.clone(), global_memory.clone()) => res2,
        }
    }

    #[cfg(not(feature = "engine"))]
    {
        // Run server with just the Privy client
        // Agents will be created dynamically based on requests
        run_server(privy_client, mongo, global_memory).await
    }
}

#[cfg(not(feature = "http"))]
fn main() {
    tracing::warn!("This binary requires the 'http' feature");
}
