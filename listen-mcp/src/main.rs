use clap::{Parser, ValueEnum};
use listen_mcp::tools::{FetchPrice, FetchPriceChart, FetchTokenMetadata, FetchTopTokens};
use mcp_core::{
    server::Server,
    transport::{ServerSseTransport, ServerStdioTransport},
    types::ServerCapabilities,
};
use serde_json::json;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Transport type to use
    #[arg(value_enum, default_value_t = TransportType::Sse)]
    transport: TransportType,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum TransportType {
    Stdio,
    Sse,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if matches!(cli.transport, TransportType::Stdio) {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let server_protocol = Server::builder("listen-mcp".to_string(), "1.0".to_string())
        .capabilities(ServerCapabilities {
            tools: Some(json!({
                "listChanged": false,
            })),
            ..Default::default()
        })
        .register_tool(FetchTopTokens::tool(), FetchTopTokens::call())
        .register_tool(FetchPrice::tool(), FetchPrice::call())
        .register_tool(FetchPriceChart::tool(), FetchPriceChart::call())
        .register_tool(FetchTokenMetadata::tool(), FetchTokenMetadata::call())
        .build();

    match cli.transport {
        TransportType::Stdio => {
            let transport = ServerStdioTransport::new(server_protocol);
            Server::start(transport).await
        }
        TransportType::Sse => {
            let transport = ServerSseTransport::new("127.0.0.1".to_string(), 3000, server_protocol);
            Server::start(transport).await
        }
    }
}
