use crate::blockhash::update_latest_blockhash;
use crate::constants::JITO_TIP_PUBKEY;
use crate::jito::SearcherClient;
use crate::pump::{self, PumpBuyRequest};
use crate::util::{env, healthz};
use actix_web::web::Data;
use actix_web::{get, post, web::Json, App, Error, HttpResponse, HttpServer};
use futures_util::StreamExt;
use jito_protos::searcher::SubscribeBundleResultsRequest;
use jito_searcher_client::{get_searcher_client, send_bundle_no_wait};
use log::info;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PumpAppState {
    pub wallet: Arc<Mutex<Keypair>>,
    pub searcher_client: Arc<Mutex<SearcherClient>>,
    pub latest_blockhash: Arc<Mutex<Hash>>,
}

#[get("/blockhash")]
#[timed::timed(duration(printer = "info!"))]
pub async fn get_blockhash(state: Data<PumpAppState>) -> HttpResponse {
    let blockhash = state.latest_blockhash.lock().await;
    HttpResponse::Ok().json(json!({
        "blockhash": blockhash.to_string()
    }))
}

#[post("/pump-buy")]
#[timed::timed(duration(printer = "info!"))]
pub async fn handle_pump_buy(
    pump_buy_request: Json<PumpBuyRequest>,
    state: Data<PumpAppState>,
) -> Result<HttpResponse, Error> {
    info!(
        "handling pump buy req {}",
        serde_json::to_string_pretty(&pump_buy_request)?
    );
    let lamports = 100_000;
    let tip = 100_000;
    let mint = pump_buy_request.mint;
    let pump_buy_request = pump_buy_request.clone();
    let token_amount = pump::get_token_amount(
        pump_buy_request.virtual_sol_reserves,
        pump_buy_request.virtual_token_reserves,
        pump_buy_request.real_token_reserves,
        lamports,
    )?;
    let token_amount = (token_amount as f64 * 0.7) as u64;
    let wallet = state.wallet.lock().await;
    let mut searcher_client = state.searcher_client.lock().await;
    let latest_blockhash = state.latest_blockhash.lock().await;
    let mut ixs = pump::_make_buy_ixs(
        wallet.pubkey(),
        pump_buy_request.mint,
        pump_buy_request.bonding_curve,
        pump_buy_request.associated_bonding_curve,
        token_amount,
        lamports,
    )?;
    ixs.push(transfer(&wallet.pubkey(), &JITO_TIP_PUBKEY, tip));
    let swap_tx =
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            ixs.as_slice(),
            Some(&wallet.pubkey()),
            &[&*wallet],
            *latest_blockhash,
        ));
    let start = std::time::Instant::now();
    let res = send_bundle_no_wait(&[swap_tx], &mut searcher_client)
        .await
        .expect("send bundle no wait");
    let elapsed = start.elapsed();
    info!("Bundle sent in {:?}", elapsed);
    info!("Bundle sent. UUID: {}", res.into_inner().uuid);
    Ok(HttpResponse::Ok().json(json!({
    "status": format!(
        "OK, trigerred buy of {}",
        mint.to_string())
    })))
}

pub async fn run_pump_service() -> std::io::Result<()> {
    // keep all of the state in the app state not to re-init
    let wallet = Arc::new(Mutex::new(
        Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
            .expect("read fund keypair"),
    ));
    let auth =
        Arc::new(Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap());
    let searcher_client = Arc::new(Mutex::new(
        get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
            .await
            .expect("makes searcher client"),
    ));

    // keep a stream for bundle results
    // TODO hopefully this doesn't deadlock
    let mut bundle_results_stream = searcher_client
        .lock()
        .await
        .subscribe_bundle_results(SubscribeBundleResultsRequest {})
        .await
        .expect("subscribe bundle results")
        .into_inner();

    let app_state = Data::new(PumpAppState {
        wallet,
        searcher_client,
        latest_blockhash: Arc::new(Mutex::new(Hash::default())),
    });

    // poll for latest blockhash to trim 200ms
    let rpc_client = Arc::new(RpcClient::new(env("RPC_URL")));
    tokio::spawn(update_latest_blockhash(
        rpc_client.clone(),
        app_state.latest_blockhash.clone(),
    ));

    // poll for bundle results
    tokio::spawn(async move {
        while let Some(res) = bundle_results_stream.next().await {
            info!("Received bundle result: {:?}", res);
        }
    });

    info!("Running pump service on 6969");
    HttpServer::new(move || {
        App::new()
            .service(handle_pump_buy)
            .service(get_blockhash)
            .service(healthz)
            .app_data(app_state.clone())
    })
    .bind(("0.0.0.0", 6969))?
    .run()
    .await
}
