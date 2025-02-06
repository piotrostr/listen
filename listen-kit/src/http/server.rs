use crate::wallet_manager::WalletManager;
use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;

use super::routes::{auth, healthz, stream};
use super::state::AppState;

pub async fn run_server(
    #[cfg(feature = "solana")] solana_agent: Agent<CompletionModel>,
    #[cfg(feature = "evm")] evm_agent: Agent<CompletionModel>,
    wallet_manager: WalletManager,
    omni_agent: Agent<CompletionModel>,
) -> std::io::Result<()> {
    let mut builder = AppState::builder().with_wallet_manager(wallet_manager);

    builder = builder.with_omni_agent(omni_agent);

    #[cfg(feature = "solana")]
    {
        builder = builder.with_solana_agent(solana_agent);
    }

    #[cfg(feature = "evm")]
    {
        builder = builder.with_evm_agent(evm_agent);
    }

    let state =
        web::Data::new(builder.build().expect("Failed to build AppState"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(healthz)
            .service(web::scope("/v1").service(stream).service(auth))
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
