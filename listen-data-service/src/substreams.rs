use anyhow::{anyhow, Error};
use async_stream::try_stream;
use futures::{Stream, StreamExt};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tokio_retry::strategy::ExponentialBackoff;

use crate::pb::sf::substreams::rpc::v2::{
    response::Message, stream_client::StreamClient, BlockScopedData, BlockUndoSignal, Request,
    Response,
};
use crate::pb::sf::substreams::v1::Modules;
use std::fmt::Display;

use http::{uri::Scheme, Uri};
use tonic::{
    codegen::http,
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
};

pub enum BlockResponse {
    New(BlockScopedData),
    Undo(BlockUndoSignal),
}

pub struct SubstreamsStream {
    stream: Pin<Box<dyn Stream<Item = Result<BlockResponse, Error>> + Send>>,
}

impl SubstreamsStream {
    pub fn new(
        endpoint: Arc<SubstreamsEndpoint>,
        cursor: Option<String>,
        modules: Option<Modules>,
        output_module_name: String,
        start_block: i64,
        end_block: u64,
    ) -> Self {
        SubstreamsStream {
            stream: Box::pin(stream_blocks(
                endpoint,
                cursor,
                modules,
                output_module_name,
                start_block,
                end_block,
            )),
        }
    }
}

// Create the Stream implementation that streams blocks with auto-reconnection.
fn stream_blocks(
    endpoint: Arc<SubstreamsEndpoint>,
    cursor: Option<String>,
    modules: Option<Modules>,
    output_module_name: String,
    start_block_num: i64,
    stop_block_num: u64,
) -> impl Stream<Item = Result<BlockResponse, Error>> {
    let mut latest_cursor = cursor.unwrap_or_default();
    let mut backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));
    let mut last_progress_report = Instant::now();

    try_stream! {
        loop {
            println!("Blockstreams disconnected, connecting (endpoint {}, start block {}, stop block {}, cursor {})",
                &endpoint,
                start_block_num,
                stop_block_num,
                &latest_cursor
            );

            let result = endpoint.clone().substreams(Request {
                start_block_num,
                start_cursor: latest_cursor.clone(),
                stop_block_num,
                final_blocks_only: false,
                modules: modules.clone(),
                output_module: output_module_name.clone(),
                production_mode: true,
                debug_initial_store_snapshot_for_modules: vec![],
            }).await;

            match result {
                Ok(stream) => {
                    println!("Blockstreams connected");

                    let mut encountered_error = false;
                    for await response in stream{
                        match process_substreams_response(response, &mut last_progress_report).await {
                            BlockProcessedResult::BlockScopedData(block_scoped_data) => {
                                // Reset backoff because we got a good value from the stream
                                backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

                                let cursor = block_scoped_data.cursor.clone();
                                yield BlockResponse::New(block_scoped_data);

                                latest_cursor = cursor;
                            },
                            BlockProcessedResult::BlockUndoSignal(block_undo_signal) => {
                                // Reset backoff because we got a good value from the stream
                                backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

                                let cursor = block_undo_signal.last_valid_cursor.clone();
                                yield BlockResponse::Undo(block_undo_signal);

                                latest_cursor = cursor;
                            },
                            BlockProcessedResult::Skip() => {},
                            BlockProcessedResult::TonicError(status) => {
                                // Unauthenticated errors are not retried, we forward the error back to the
                                // stream consumer which handles it
                                if status.code() == tonic::Code::Unauthenticated {
                                    return Err(anyhow::Error::new(status.clone()))?;
                                }

                                println!("Received tonic error {:#}", status);
                                encountered_error = true;
                                break;
                            },
                        }
                    }

                    if !encountered_error {
                        println!("Stream completed, reached end block");
                        return
                    }
                },
                Err(e) => {
                    // We failed to connect and will try again; this is another
                    // case where we actually _want_ to back off in case we keep
                    // having connection errors.

                    println!("Unable to connect to endpoint: {:#}", e);
                }
            }

            // If we reach this point, we must wait a bit before retrying
            if let Some(duration) = backoff.next() {
                sleep(duration).await
            } else {
                return Err(anyhow!("backoff requested to stop retrying, quitting"))?;
            }
        }
    }
}

enum BlockProcessedResult {
    Skip(),
    BlockScopedData(BlockScopedData),
    BlockUndoSignal(BlockUndoSignal),
    TonicError(tonic::Status),
}

async fn process_substreams_response(
    result: Result<Response, tonic::Status>,
    _last_progress_report: &mut Instant,
) -> BlockProcessedResult {
    let response = match result {
        Ok(v) => v,
        Err(e) => return BlockProcessedResult::TonicError(e),
    };

    match response.message {
        Some(Message::Session(session)) => {
            println!("Received session message (Trace ID {})", &session.trace_id);
            BlockProcessedResult::Skip()
        }
        Some(Message::BlockScopedData(block_scoped_data)) => {
            BlockProcessedResult::BlockScopedData(block_scoped_data)
        }
        Some(Message::BlockUndoSignal(block_undo_signal)) => {
            BlockProcessedResult::BlockUndoSignal(block_undo_signal)
        }
        Some(Message::Progress(progress)) => {
            println!("Received progress message: {:#?}", progress);
            // The `ModulesProgress` messages goal is to report active parallel processing happening
            // either to fill up backward (relative to your request's start block) some missing state
            // or pre-process forward blocks (again relative).
            //
            // You could log that in trace or accumulate to push as metrics. Here a snippet of code
            // that prints progress to standard out. If your `BlockScopedData` messages seems to never
            // arrive in production mode, it's because progresses is happening but not yet for the output
            // module you requested.
            //
            // let progresses: Vec<_> = progress
            //     .modules
            //     .iter()
            //     .filter_map(|module| {
            //         use crate::pb::sf::substreams::rpc::v2::module_progress::Type;

            //         if let Type::ProcessedRanges(range) = module.r#type.as_ref().unwrap() {
            //             Some(format!(
            //                 "{} @ [{}]",
            //                 module.name,
            //                 range
            //                     .processed_ranges
            //                     .iter()
            //                     .map(|x| x.to_string())
            //                     .collect::<Vec<_>>()
            //                     .join(", ")
            //             ))
            //         } else {
            //             None
            //         }
            //     })
            //     .collect();

            // println!("Progess {}", progresses.join(", "));

            BlockProcessedResult::Skip()
        }
        None => {
            println!("Got None on substream message");
            BlockProcessedResult::Skip()
        }
        _ => BlockProcessedResult::Skip(),
    }
}

impl Stream for SubstreamsStream {
    type Item = Result<BlockResponse, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

#[derive(Clone, Debug)]
pub struct SubstreamsEndpoint {
    pub uri: String,
    pub token: Option<String>,
    channel: Channel,
}

impl Display for SubstreamsEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.uri.as_str(), f)
    }
}

impl SubstreamsEndpoint {
    pub async fn new<S: AsRef<str>>(url: S, token: Option<String>) -> Result<Self, anyhow::Error> {
        let uri = url
            .as_ref()
            .parse::<Uri>()
            .expect("the url should have been validated by now, so it is a valid Uri");

        let endpoint = match uri.scheme().unwrap_or(&Scheme::HTTP).as_str() {
            "http" => Channel::builder(uri),
            "https" => Channel::builder(uri)
                .tls_config(ClientTlsConfig::new().with_native_roots())
                .expect("TLS config on this host is invalid"),
            _ => panic!("invalid uri scheme for firehose endpoint"),
        }
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Some(Duration::from_secs(30)));

        let uri = endpoint.uri().to_string();
        let channel = endpoint.connect_lazy();

        Ok(SubstreamsEndpoint {
            uri,
            channel,
            token,
        })
    }

    pub async fn substreams(
        self: Arc<Self>,
        request: Request,
    ) -> Result<tonic::Streaming<Response>, anyhow::Error> {
        println!("Connecting to substreams endpoint {}", self.uri);
        let token_metadata: Option<MetadataValue<tonic::metadata::Ascii>> = match self.token.clone()
        {
            Some(token) => Some(format!("Bearer {}", token).as_str().try_into()?),

            None => None,
        };

        let mut client = StreamClient::with_interceptor(
            self.channel.clone(),
            move |mut r: tonic::Request<()>| {
                if let Some(ref t) = token_metadata {
                    r.metadata_mut().insert("authorization", t.clone());
                }

                Ok(r)
            },
        );

        let response_stream = client.blocks(request).await?;
        let block_stream = response_stream.into_inner();

        Ok(block_stream)
    }
}
