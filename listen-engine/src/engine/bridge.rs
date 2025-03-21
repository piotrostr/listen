use std::collections::HashSet;

use crate::engine::{
    pipeline::{PipelineStep, Status},
    Engine, EngineError, Pipeline,
};
use uuid::Uuid;

impl Engine {
    pub async fn delete_pipeline(
        &self,
        user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<(), EngineError> {
        if let Err(e) = self
            .redis
            .delete_pipeline(user_id, &pipeline_id.to_string())
            .await
        {
            return Err(EngineError::DeletePipelineError(e));
        }

        Ok(())
    }

    pub async fn get_all_pipelines_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Pipeline>, EngineError> {
        match self
            .redis
            .get_all_pipelines_for_user(user_id)
            .await
            .map_err(EngineError::RedisClientError)
        {
            Ok(pipelines) => Ok(pipelines),
            Err(e) => {
                tracing::error!("Error getting all pipelines for user: {}", e);
                Err(e)
            }
        }
    }

    pub async fn cancel_pipeline(
        &self,
        user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<(), EngineError> {
        let mut pipeline = match self
            .redis
            .get_pipeline(user_id, &pipeline_id.to_string())
            .await
        {
            Ok(Some(pipeline)) => pipeline,
            Ok(None) => return Err(EngineError::PipelineNotFound(pipeline_id.to_string())),
            Err(e) => return Err(EngineError::RedisClientError(e)),
        };

        if pipeline.user_id != user_id {
            return Err(EngineError::Unauthorized);
        }

        pipeline.status = Status::Cancelled;

        for step in pipeline.steps.values_mut() {
            if matches!(step.status, Status::Pending) {
                step.status = Status::Cancelled;
            }
        }

        let assets_mentioned = self.extract_assets(&pipeline);

        for asset in assets_mentioned {
            match self.active_pipelines.get_mut(&asset) {
                Some(mut pipeline_ids) => {
                    pipeline_ids.remove(&pipeline.id.to_string());
                    drop(pipeline_ids);
                }
                None => {
                    tracing::error!("Asset {} not found in active pipelines", asset);
                }
            }
        }

        if let Err(e) = self.redis.save_pipeline(&pipeline).await {
            return Err(EngineError::RedisClientError(e));
        }

        Ok(())
    }

    pub async fn cancel_step(
        &self,
        user_id: &str,
        pipeline_id: Uuid,
        step_id: Uuid,
    ) -> Result<(), EngineError> {
        let mut pipeline = match self
            .redis
            .get_pipeline(user_id, &pipeline_id.to_string())
            .await
        {
            Ok(Some(pipeline)) => pipeline,
            Ok(None) => return Err(EngineError::PipelineNotFound(pipeline_id.to_string())),
            Err(e) => return Err(EngineError::RedisClientError(e)),
        };

        if pipeline.user_id != user_id {
            return Err(EngineError::Unauthorized);
        }

        if let Some(step) = pipeline.steps.get_mut(&step_id) {
            if matches!(step.status, Status::Pending) {
                step.status = Status::Cancelled;

                let mut to_cancel: Vec<PipelineStep> = vec![step.clone()];
                let mut queue = vec![step_id];

                while let Some(current_id) = queue.pop() {
                    if let Some(current_step) = pipeline.steps.get(&current_id) {
                        for next_id in &current_step.next_steps {
                            if let Some(next_step) = pipeline.steps.get(next_id) {
                                if matches!(next_step.status, Status::Pending) {
                                    to_cancel.push(next_step.clone());
                                    queue.push(*next_id);
                                }
                            }
                        }
                    }
                }

                for cancel_id in to_cancel.iter() {
                    if let Some(step) = pipeline.steps.get_mut(&cancel_id.id) {
                        step.status = Status::Cancelled;
                    }
                }

                let mut assets_mentioned: HashSet<String> = HashSet::new();
                for step in to_cancel.iter() {
                    self.collect_assets_from_condition(&step.conditions, &mut assets_mentioned);
                }

                for asset in assets_mentioned {
                    match self.active_pipelines.get_mut(&asset) {
                        Some(mut pipeline_ids) => {
                            pipeline_ids.remove(&pipeline.id.to_string());
                            drop(pipeline_ids);
                        }
                        None => {
                            tracing::error!("Asset {} not found in active pipelines", asset);
                        }
                    }
                }

                if let Err(e) = self.redis.save_pipeline(&pipeline).await {
                    return Err(EngineError::RedisClientError(e));
                }

                return Ok(());
            } else {
                return Err(EngineError::StepNotCancellable);
            }
        } else {
            return Err(EngineError::StepNotFound(step_id.to_string()));
        }
    }
}
