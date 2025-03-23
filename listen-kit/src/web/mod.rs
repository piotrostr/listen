pub mod client;
pub mod crawl;
pub mod search;

#[derive(thiserror::Error, Debug)]
pub enum WebError {
    #[error("[Web] {0}")]
    ExaClientError(#[from] client::ExaClientError),

    #[error("[Web] {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub struct Web {
    client: client::ExaClient,
}

impl Web {
    pub fn from_env() -> Result<Self, WebError> {
        let client = client::ExaClient::from_env()?;
        Ok(Self { client })
    }
}
