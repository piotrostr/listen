#[cfg(feature = "http")]
use {
    listen_kit::http::server::run_server,
    listen_kit::mongo::MongoClient,
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

    // Run server with just the Privy client
    // Agents will be created dynamically based on requests
    run_server(privy_client, mongo).await
}

#[cfg(not(feature = "http"))]
fn main() {
    println!("This binary requires the 'http' feature");
}
