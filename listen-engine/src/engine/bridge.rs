use std::sync::Arc;

use crate::engine::{Engine, EngineError, Pipeline};
use tokio::sync::RwLock;
use uuid::Uuid;

impl Engine {
    pub async fn add_pipeline(&self, pipeline: &Pipeline) -> Result<(), EngineError> {
        // Extract all assets mentioned in pipeline conditions
        let assets = self.extract_assets(pipeline).await;

        // Update asset subscriptions
        for asset in assets {
            self.asset_subscriptions
                .entry(asset)
                .or_default()
                .insert(pipeline.id);
        }

        // Clone pipeline before inserting
        self.active_pipelines
            .insert(pipeline.id, Arc::new(RwLock::new(pipeline.clone())));

        Ok(())
    }

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
        self.active_pipelines.remove(&pipeline_id);

        Ok(())
    }

    pub async fn get_pipeline(
        &self,
        _user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<Pipeline, EngineError> {
        let pipeline = self.active_pipelines.get(&pipeline_id);
        let pipeline = match pipeline {
            Some(pipeline) => pipeline.read().await.clone(),
            None => {
                return Err(EngineError::GetPipelineError(format!(
                    "Pipeline not found: {}",
                    pipeline_id
                )))
            }
        };
        Ok(pipeline)
    }

    pub async fn get_all_pipelines_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Pipeline>, EngineError> {
        println!("getting all pipelines for user: {}", user_id);
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
}
