#[derive(Debug, thiserror::Error)]
pub enum ApprovalsError {
    #[error("Invalid CAIP2: {0}")]
    InvalidCaip2(String),

    #[error("Unsupported chain ID: {0}")]
    UnsupportedChainId(String),

    #[error("Failed to get Alchemy API key")]
    FailedToGetAlchemyApiKey,

    #[error("Failed to get allowance: {0}")]
    FailedToGetAllowance(reqwest::Error),

    #[error("Failed to estimate gas: {0}")]
    FailedToEstimateGas(String),

    #[error("Invalid chain ID: {0}")]
    InvalidChainId(String),
}
