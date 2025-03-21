use crate::engine::error::EngineError;
use crate::engine::pipeline::Pipeline;
use std::sync::Arc;

use privy::Privy;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum EngineMessage {
    AddPipeline {
        pipeline: Pipeline,
        response_tx: oneshot::Sender<Result<String, EngineError>>,
    },
    GetPipeline {
        user_id: String,
        pipeline_id: Uuid,
        response_tx: oneshot::Sender<Result<Pipeline, EngineError>>,
    },
    DeletePipeline {
        user_id: String,
        pipeline_id: Uuid,
        response_tx: oneshot::Sender<Result<(), EngineError>>,
    },
    GetAllPipelinesByUser {
        user_id: String,
        response_tx: oneshot::Sender<Result<Vec<Pipeline>, EngineError>>,
    },
    CancelPipeline {
        user_id: String,
        pipeline_id: Uuid,
        response_tx: oneshot::Sender<Result<(), EngineError>>,
    },
    CancelStep {
        user_id: String,
        pipeline_id: Uuid,
        step_id: Uuid,
        response_tx: oneshot::Sender<Result<(), EngineError>>,
    },
}

pub struct AppState {
    pub engine_bridge_tx: mpsc::Sender<EngineMessage>,
    pub privy: Arc<Privy>,
}
