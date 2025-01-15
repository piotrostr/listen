use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use jito_protos::{
    auth::{auth_service_client::AuthServiceClient, Role},
    bundle::{
        bundle_result::Result as BundleResultType, rejected::Reason, Accepted, Bundle,
        BundleResult, InternalError, SimulationFailure, StateAuctionBidRejected,
        WinningBatchBidRejected,
    },
    convert::proto_packet_from_versioned_tx,
    searcher::{
        searcher_service_client::SearcherServiceClient, SendBundleRequest, SendBundleResponse,
    },
};
use log::{info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signature},
    transaction::VersionedTransaction,
};
use thiserror::Error;
use tokio::time::timeout;
use tonic::{
    codegen::InterceptedService,
    transport,
    transport::{Channel, Endpoint},
    Response, Status, Streaming,
};

use crate::token_authenticator::ClientInterceptor;

pub mod token_authenticator;

#[derive(Debug, Error)]
pub enum BlockEngineConnectionError {
    #[error("transport error {0}")]
    TransportError(#[from] transport::Error),
    #[error("client error {0}")]
    ClientError(#[from] Status),
}

#[derive(Debug, Error)]
pub enum BundleRejectionError {
    #[error("bundle lost state auction, auction: {0}, tip {1} lamports")]
    StateAuctionBidRejected(String, u64),
    #[error("bundle won state auction but failed global auction, auction {0}, tip {1} lamports")]
    WinningBatchBidRejected(String, u64),
    #[error("bundle simulation failure on tx {0}, message: {1:?}")]
    SimulationFailure(String, Option<String>),
    #[error("internal error {0}")]
    InternalError(String),
}

pub type BlockEngineConnectionResult<T> = Result<T, BlockEngineConnectionError>;

pub async fn get_searcher_client(
    block_engine_url: &str,
    auth_keypair: &Arc<Keypair>,
) -> BlockEngineConnectionResult<
    SearcherServiceClient<InterceptedService<Channel, ClientInterceptor>>,
> {
    let auth_channel = create_grpc_channel(block_engine_url).await?;
    let client_interceptor = ClientInterceptor::new(
        AuthServiceClient::new(auth_channel),
        auth_keypair,
        Role::Searcher,
    )
    .await?;

    let searcher_channel = create_grpc_channel(block_engine_url).await?;
    let searcher_client =
        SearcherServiceClient::with_interceptor(searcher_channel, client_interceptor);
    Ok(searcher_client)
}

pub async fn create_grpc_channel(url: &str) -> BlockEngineConnectionResult<Channel> {
    let mut endpoint = Endpoint::from_shared(url.to_string()).expect("invalid url");
    if url.starts_with("https") {
        endpoint = endpoint.tls_config(tonic::transport::ClientTlsConfig::new())?;
    }
    Ok(endpoint.connect().await?)
}

pub async fn send_bundle_with_confirmation(
    transactions: &[VersionedTransaction],
    rpc_client: &RpcClient,
    searcher_client: &mut SearcherServiceClient<InterceptedService<Channel, ClientInterceptor>>,
    bundle_results_subscription: &mut Streaming<BundleResult>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_signatures: Vec<Signature> =
        transactions.iter().map(|tx| tx.signatures[0]).collect();

    let result = send_bundle_no_wait(transactions, searcher_client).await?;

    // grab uuid from block engine + wait for results
    let uuid = result.into_inner().uuid;
    info!("Bundle sent. UUID: {:?}", uuid);

    info!("Waiting for 5 seconds to hear results...");
    let mut time_left = 5000;
    while let Ok(Some(Ok(results))) = timeout(
        Duration::from_millis(time_left),
        bundle_results_subscription.next(),
    )
    .await
    {
        let instant = Instant::now();
        info!("bundle results: {:?}", results);
        match results.result {
            Some(BundleResultType::Accepted(Accepted {
                slot: _s,
                validator_identity: _v,
            })) => {
                info!("accepted into slot {} by {}", _s, _v);
                return Ok(());
            }
            Some(BundleResultType::Rejected(rejected)) => {
                match rejected.reason {
                    Some(Reason::WinningBatchBidRejected(WinningBatchBidRejected {
                        auction_id,
                        simulated_bid_lamports,
                        msg: _,
                    })) => {
                        return Err(Box::new(BundleRejectionError::WinningBatchBidRejected(
                            auction_id,
                            simulated_bid_lamports,
                        )))
                    }
                    Some(Reason::StateAuctionBidRejected(StateAuctionBidRejected {
                        auction_id,
                        simulated_bid_lamports,
                        msg: _,
                    })) => {
                        return Err(Box::new(BundleRejectionError::StateAuctionBidRejected(
                            auction_id,
                            simulated_bid_lamports,
                        )))
                    }
                    Some(Reason::SimulationFailure(SimulationFailure { tx_signature, msg })) => {
                        return Err(Box::new(BundleRejectionError::SimulationFailure(
                            tx_signature,
                            msg,
                        )))
                    }
                    Some(Reason::InternalError(InternalError { msg })) => {
                        return Err(Box::new(BundleRejectionError::InternalError(msg)))
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        time_left -= instant.elapsed().as_millis() as u64;
    }

    let futs: Vec<_> = bundle_signatures
        .iter()
        .map(|sig| {
            rpc_client.get_signature_status_with_commitment(sig, CommitmentConfig::processed())
        })
        .collect();
    let results = futures_util::future::join_all(futs).await;
    if !results.iter().all(|r| matches!(r, Ok(Some(Ok(()))))) {
        warn!("Transactions in bundle did not land");
        return Err(Box::new(BundleRejectionError::InternalError(
            "Searcher service did not provide bundle status in time".into(),
        )));
    }
    info!("Bundle landed successfully");
    for sig in bundle_signatures.iter() {
        info!("https://solscan.io/tx/{}", sig);
    }
    Ok(())
}

pub async fn send_bundle_no_wait(
    transactions: &[VersionedTransaction],
    searcher_client: &mut SearcherServiceClient<InterceptedService<Channel, ClientInterceptor>>,
) -> Result<Response<SendBundleResponse>, Status> {
    // convert them to packets + send over
    let packets: Vec<_> = transactions
        .iter()
        .map(proto_packet_from_versioned_tx)
        .collect();

    searcher_client
        .send_bundle(SendBundleRequest {
            bundle: Some(Bundle {
                header: None,
                packets,
            }),
        })
        .await
}