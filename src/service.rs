use crate::api_docs::ApiDocs;
use crate::blockhash::update_latest_blockhash;
use crate::handlers::{
    handle_balance, handle_pump_buy, handle_pump_sell, handle_swap,
    handle_token_balance,
};
use crate::state::ServiceState;
use crate::util::{env, healthz};
use actix_web::{web::Data, App, HttpServer};
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{hash::Hash, signature::Keypair, signer::EncodableKey};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct ListenService {
    port: u16,
    state: Arc<ServiceState>,
}

impl ListenService {
    pub fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let wallet = Arc::new(Mutex::new(
            Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .map_err(|_| "read fund keypair")?,
        ));

        let rpc_client = Arc::new(RpcClient::new(env("RPC_URL")));

        let state = Arc::new(ServiceState {
            wallet,
            rpc_client,
            latest_blockhash: Arc::new(Mutex::new(Hash::default())),
        });

        Ok(Self { port, state })
    }

    pub async fn start(&self) -> std::io::Result<()> {
        // Clone state for use in HTTP server
        let state = self.state.clone();

        // Start blockhash updater
        let rpc_client = self.state.rpc_client.clone();
        let latest_blockhash = self.state.latest_blockhash.clone();
        tokio::spawn(update_latest_blockhash(rpc_client, latest_blockhash));

        info!("Running unified listen service on port {}", self.port);

        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(state.clone()))
                .service(handle_swap)
                .service(handle_balance)
                .service(handle_pump_buy)
                .service(handle_pump_sell)
                .service(handle_token_balance)
                .service(healthz)
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", ApiDocs::openapi()),
                )
        })
        .bind(("0.0.0.0", self.port))?
        .run()
        .await
    }
}

// Main entry point
pub async fn run_listen_service() -> std::io::Result<()> {
    let service = ListenService::new(6969).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NetworkUnreachable,
            "Failed to create listen service",
        )
    })?;
    service.start().await
}
