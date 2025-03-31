use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    common::spawn_with_signer,
    reasoning_loop::{Model, ReasoningLoop, StreamResponse},
    signer::TransactionSigner,
};
use privy::util::base64encode;

// FIXME this has to break as the signal is sent to cancel the request!

/// Delegate a task to a specific agent and handle the response
pub async fn delegate_to_agent(
    prompt: String,
    agent: Model,
    agent_type: String,
    signer: Arc<dyn TransactionSigner>,
    with_stdout: bool,
) -> Result<String> {
    let reasoning_loop = ReasoningLoop::new(agent).with_stdout(with_stdout);

    // Get the parent agent's stream channel from the task-local variable
    let parent_tx = crate::reasoning_loop::get_current_stream_channel().await;
    println!("parent_tx: {:?}", parent_tx);

    // Create a channel for collecting the agent's output
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamResponse>(1024);
    let res = Arc::new(RwLock::new(String::new()));
    let res_ptr = res.clone();

    let reader_handle = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            res_ptr.write().await.push_str(&format!(
                "<content>{}</content>",
                base64encode(
                    &serde_json::to_vec(&response).unwrap_or_default()
                )
            ));

            // Forward to parent if available, as a NestedAgentOutput
            if let Some(parent_tx) = &parent_tx {
                let nested_output = StreamResponse::NestedAgentOutput {
                    agent_type: agent_type.clone(),
                    content: format!(
                        "<content>{}</content>",
                        base64encode(
                            &serde_json::to_vec(&response)
                                .unwrap_or_default()
                        )
                    ),
                };
                let _ = parent_tx.send(nested_output).await;
            }

            // Still log to console if needed
            if with_stdout {
                if let StreamResponse::Message(msg) = &response {
                    print!("{}", msg);
                }
            }
        }
    });

    let loop_handle = spawn_with_signer(signer, || async move {
        reasoning_loop.stream(prompt, vec![], Some(tx)).await
    })
    .await;

    let _ = tokio::try_join!(reader_handle, loop_handle);

    let response = res.read().await.to_string();

    Ok(response)
}
