use crate::engine::evaluator::EvaluatorError;
use crate::engine::order::SwapOrderError;
use crate::redis::client::RedisClientError;
use crate::redis::subscriber::RedisSubscriberError;
use privy::config::PrivyConfigError;
use privy::PrivyError;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("[Engine] Failed to add pipeline: {0}")]
    AddPipelineError(RedisClientError),

    #[error("[Engine] Pipeline not found: {0}")]
    PipelineNotFound(String),

    #[error("[Engine] Failed to save pipeline: {0}")]
    SavePipelineError(RedisClientError),

    #[error("[Engine] Failed to delete pipeline: {0}")]
    DeletePipelineError(RedisClientError),

    #[error("[Engine] Failed to get pipeline: {0}")]
    GetPipelineError(String),

    #[error("[Engine] Failed to evaluate pipeline: {0}")]
    EvaluatePipelineError(EvaluatorError),

    #[error("[Engine] Failed to extract assets: {0}")]
    ExtractAssetsError(anyhow::Error),

    #[error("[Engine] Failed to handle price update: {0}")]
    HandlePriceUpdateError(anyhow::Error),

    #[error("[Engine] Transaction error: {0}")]
    TransactionError(privy::tx::PrivyTransactionError),

    #[error("[Engine] Swap order error: {0}")]
    SwapOrderError(SwapOrderError),

    #[error("[Engine] Redis client error: {0}")]
    RedisClientError(RedisClientError),

    #[error("[Engine] Redis subscriber error: {0}")]
    RedisSubscriberError(RedisSubscriberError),

    #[error("[Engine] Privy error: {0}")]
    PrivyError(PrivyError),

    #[error("[Engine] Privy config error: {0}")]
    PrivyConfigError(PrivyConfigError),

    #[error("[Engine] Blockhash cache error: {0}")]
    BlockhashCacheError(blockhash_cache::BlockhashCacheError),

    #[error("[Engine] Inject blockhash error: {0}")]
    InjectBlockhashError(anyhow::Error),

    #[error("[Engine] Approvals error: {0}")]
    ApprovalsError(evm_approvals::ApprovalsError),

    #[error("[Engine] EVM Wallet not available")]
    EVMWalletNotAvailable,

    #[error("[Engine] Solana Wallet not available")]
    SolanaWalletNotAvailable,

    #[error("[Engine] Step not found: {0}")]
    StepNotFound(String),

    #[error("[Engine] Step not cancellable")]
    StepNotCancellable,

    #[error("[Engine] Unauthorized")]
    Unauthorized,
}
