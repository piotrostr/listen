use anyhow::Result;
use listen_data_service::pipeline::make_pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    let mut pipeline = make_pipeline()?;

    pipeline.run().await?;

    Ok(())
}
