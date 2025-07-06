#[cfg(feature = "http")]
use {
    listen_kit::http::server::run_server,
    listen_mongo::MongoClient,
    privy::{config::PrivyConfig, Privy},
};

#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Privy client
    let privy_client =
        Privy::new(PrivyConfig::from_env().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?);

    let mongo = MongoClient::from_env()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    #[cfg(feature = "engine")]
    {
        tokio::select! {
            res1 = listen_engine::server::run() => res1,
            res2 = run_server(privy_client, mongo) => res2,
        }
    }

    #[cfg(not(feature = "engine"))]
    {
        // Run server with just the Privy client
        // Agents will be created dynamically based on requests
        run_server(privy_client, mongo).await
    }
}

#[cfg(not(feature = "http"))]
fn main() {
    tracing::warn!("This binary requires the 'http' feature");
}
