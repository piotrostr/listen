use crate::engine::{Engine, EngineError, Pipeline};
use uuid::Uuid;

impl Engine {
    pub async fn add_pipeline(&self, pipeline: Pipeline) -> Result<(), EngineError> {
        // Add to engine's state
        let mut active_pipelines = self.active_pipelines.write().await;
        let mut asset_subscriptions = self.asset_subscriptions.write().await;

        // Extract all assets mentioned in pipeline conditions
        let assets = self.extract_assets(&pipeline).await;

        // Update asset subscriptions
        for asset in assets {
            asset_subscriptions
                .entry(asset)
                .or_default()
                .insert(pipeline.id);
        }

        // Clone pipeline before inserting
        let mut pipeline_clone = pipeline.clone();
        active_pipelines.insert(pipeline.id, pipeline);
        drop(active_pipelines); // Release the write lock

        // Execute any immediate steps
        self.evaluate_pipeline(&mut pipeline_clone).await?;

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
        let mut active_pipelines = self.active_pipelines.write().await;
        active_pipelines.remove(&pipeline_id);

        Ok(())
    }

    pub async fn get_pipeline(
        &self,
        _user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<Pipeline, EngineError> {
        let active_pipelines = self.active_pipelines.read().await;
        active_pipelines.get(&pipeline_id).cloned().ok_or_else(|| {
            EngineError::GetPipelineError(format!("Pipeline not found: {}", pipeline_id))
        })
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
}
