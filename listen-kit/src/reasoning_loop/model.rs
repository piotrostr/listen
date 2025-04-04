use anyhow::Result;
use rig::message::Message;
use rig::tool::ToolSetError;
use rig::{
    completion::CompletionError,
    streaming::{StreamingCompletion, StreamingResult},
};

use crate::reasoning_loop::Model;

impl Model {
    pub async fn stream_completion(
        &self,
        prompt: Message,
        messages: Vec<Message>,
    ) -> Result<StreamingResult, CompletionError> {
        match self {
            Model::Claude(agent) => {
                agent
                    .stream_completion(prompt, messages)
                    .await?
                    .stream()
                    .await
            }
            Model::Gemini(agent) => {
                agent
                    .stream_completion(prompt, messages)
                    .await?
                    .stream()
                    .await
            }
            Model::DeepSeek(agent) => {
                agent
                    .stream_completion(prompt, messages)
                    .await?
                    .stream()
                    .await
            }
            Model::OpenAI(agent) => {
                agent
                    .stream_completion(prompt, messages)
                    .await?
                    .stream()
                    .await
            }
            Model::OpenRouter(agent) => {
                agent
                    .stream_completion(prompt, messages)
                    .await?
                    .stream()
                    .await
            }
        }
    }

    pub async fn call_tool(
        &self,
        name: String,
        params: String,
    ) -> Result<String, ToolSetError> {
        match self {
            Model::Claude(agent) => agent.tools.call(&name, params).await,
            Model::Gemini(agent) => agent.tools.call(&name, params).await,
            Model::DeepSeek(agent) => agent.tools.call(&name, params).await,
            Model::OpenAI(agent) => agent.tools.call(&name, params).await,
            Model::OpenRouter(agent) => agent.tools.call(&name, params).await,
        }
    }
}
