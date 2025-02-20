use crate::engine::{Engine, EngineError, Pipeline};
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
}
