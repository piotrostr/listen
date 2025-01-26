use crate::api_docs::ApiDocs;
use crate::blockhash::update_latest_blockhash;
use crate::handlers::{
    handle_balance, handle_get_holdings, handle_get_pubkey, handle_pump_buy,
    handle_pump_sell, handle_swap, handle_token_balance,
};
use crate::state::ServiceState;
use crate::util::{env, healthz};
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{web::Data, App, HttpServer};
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use solana_sdk::{hash::Hash, signature::Keypair, signer::EncodableKey};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[get("/")]
async fn redirect_to_swagger() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/swagger-ui/"))
        .finish()
}

pub struct ListenService {
    port: u16,
    state: Data<ServiceState>,
}

pub fn load_keypair_from_b58_env() -> Result<Keypair, Box<dyn Error>> {
    let b58_keypair = env("FUND_KEYPAIR_BS58");
    Ok(Keypair::from_base58_string(&b58_keypair))
}

pub fn load_keypair_from_file_env() -> Result<Keypair, Box<dyn Error>> {
    let path = env("FUND_KEYPAIR_PATH");
    Keypair::read_from_file(&path)
}

impl ListenService {
    pub fn new(port: u16) -> Result<Self, Box<dyn Error>> {
        let keypair = load_keypair_from_b58_env().expect("read keypair");
        info!("Wallet address: {}", keypair.pubkey().to_string());
        let wallet = Arc::new(Mutex::new(keypair));

        let rpc_client = Arc::new(RpcClient::new(env("RPC_URL")));

        let state = Data::new(ServiceState {
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
            let cors = Cors::default()
                .allow_any_origin() // Allow all origins
                .allow_any_method() // Allow all methods
                .allow_any_header() // Allow all headers
                .max_age(3600); // Set prefligh
            App::new()
                .wrap(cors)
                .wrap(Logger::new(
                    "%{r}a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
                ))
                .app_data(state.clone())
                .service(handle_swap)
                .service(handle_get_pubkey)
                .service(handle_get_holdings)
                .service(handle_balance)
                .service(handle_pump_buy)
                .service(handle_pump_sell)
                .service(handle_token_balance)
                .service(healthz)
                .service(redirect_to_swagger)
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
    let service = ListenService::new(6969).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create listen service: {}", e),
        )
    })?;
    service.start().await
}
