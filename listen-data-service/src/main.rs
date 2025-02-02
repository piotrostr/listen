use anyhow::Result;
use futures::StreamExt;
use listen_data_service::config::Config;
use listen_data_service::package;
use listen_data_service::substreams::{BlockResponse, SubstreamsEndpoint, SubstreamsStream};
use listen_data_service::{
    load_persisted_cursor, persist_cursor, process_block_scoped_data, process_block_undo_signal,
};
use std::sync::Arc;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;

    let db = Surreal::new::<Ws>(&config.surreal.endpoint).await?;

    db.signin(Root {
        username: &config.surreal.username,
        password: &config.surreal.password,
    })
    .await?;

    db.use_ns(&config.surreal.namespace)
        .use_db(&config.surreal.database)
        .await?;

    println!("Connected to SurrealDB");

    let mut endpoint_url = config.substreams.endpoint.clone();
    if !endpoint_url.starts_with("http") {
        endpoint_url = format!("{}://{}", "https", endpoint_url);
    }

    let token = if std::env::var("SUBSTREAMS_TOKEN").is_ok() {
        Some(std::env::var("SUBSTREAMS_TOKEN")?)
    } else {
        None
    };

    let package = package::read_package(&config.modules.binary_path).await?;
    let endpoint = Arc::new(SubstreamsEndpoint::new(&endpoint_url, token).await?);

    let cursor: Option<String> = load_persisted_cursor(&db).await?;

    let mut stream = SubstreamsStream::new(
        endpoint,
        cursor,
        package.modules,
        config.substreams.output_module_name,
        config.substreams.start_block,
        config.substreams.end_block,
    );

    loop {
        match stream.next().await {
            None => break,
            Some(Ok(BlockResponse::New(data))) => {
                println!("Received block: {:#?}", data);
                process_block_scoped_data(&db, &data).await?;
                persist_cursor(&db, data.cursor).await?;
            }
            Some(Ok(BlockResponse::Undo(undo_signal))) => {
                println!("Received undo signal for block");
                process_block_undo_signal(&db, &undo_signal).await?;
                persist_cursor(&db, undo_signal.last_valid_cursor.clone()).await?;
            }
            Some(Err(e)) => {
                println!("Stream error: {:?}", e);
                if let Some(status) = e.downcast_ref::<tonic::Status>() {
                    println!("gRPC status: {:?}", status);
                    println!("gRPC metadata: {:?}", status.metadata());
                }
                return Err(e);
            }
        }
    }

    Ok(())
}
