use crate::completion::Message;
use crate::streaming::StreamingCompletion;
use crate::tools::Tools;
use anyhow::Result;

pub trait AgentTrait {
    type StreamOutput: StreamingCompletion;

    async fn stream_completion(
        &self,
        input: Message,
        messages: Vec<Message>,
    ) -> Result<Self::StreamOutput>;

    fn tools(&self) -> &Tools;
}
