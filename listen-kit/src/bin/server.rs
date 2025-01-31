#[cfg(feature = "http")]
use listen_kit::http::server::run_server;

#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use listen_kit::agent::{
        create_evm_agent, create_pump_agent, create_solana_agent,
    };
    use listen_kit::wallet_manager::config::PrivyConfig;
    use listen_kit::wallet_manager::WalletManager;

    // Initialize wallet manager
    let wallet_manager =
        WalletManager::new(PrivyConfig::from_env().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?);

    // Create agents based on enabled features
    #[cfg(feature = "solana")]
    let solana_agent = create_solana_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    #[cfg(feature = "solana")]
    let pump_fun_agent = create_pump_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    #[cfg(feature = "evm")]
    let evm_agent = create_evm_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Run server with appropriate agents based on features
    #[cfg(all(feature = "solana", feature = "evm"))]
    return run_server(
        solana_agent,
        pump_fun_agent,
        evm_agent,
        wallet_manager,
    )
    .await;

    #[cfg(all(feature = "solana", not(feature = "evm")))]
    return run_server(solana_agent, pump_fun_agent, wallet_manager).await;

    #[cfg(all(feature = "evm", not(feature = "solana")))]
    return run_server(evm_agent, wallet_manager).await;

    #[cfg(not(any(feature = "solana", feature = "evm")))]
    return run_server(wallet_manager).await;
}

#[cfg(not(feature = "http"))]
fn main() {
    println!("This binary requires the 'http' feature");
}
