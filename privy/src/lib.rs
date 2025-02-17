pub mod auth;
pub mod caip2;
pub mod config;
pub mod tx;
pub mod types;
pub mod util;

pub struct Privy {
    pub config: config::PrivyConfig,
    pub client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyError {
    #[error("Configuration error: {0}")]
    Config(config::PrivyConfigError),

    #[error("Transaction error: {0}")]
    Transaction(tx::PrivyTransactionError),

    #[error("Authentication error: {0}")]
    Auth(auth::PrivyAuthError),
}

impl Privy {
    pub fn new(config: config::PrivyConfig) -> Self {
        let client = util::create_privy_client(&config);
        Self { config, client }
    }
}
