use rig::completion::Prompt;
use rig::providers::openai;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio::sync::mpsc;

use crate::buyer::check_top_holders;
use crate::util::env;

#[derive(Debug, thiserror::Error)]
pub enum TopHoldersError {
    #[error("Invalid mint address: {0}")]
    InvalidMint(String),
    #[error("Top holders check failed: {0}")]
    CheckFailed(String),
}

#[derive(Serialize, Deserialize)]
pub struct TopHoldersOperation {
    pub mint: String,
}

#[derive(Serialize, Deserialize)]
pub struct TopHoldersOutput {
    pub percentage: f64,
    pub is_concentrated: bool,
    pub details: String,
}

#[derive(Deserialize, Serialize)]
pub struct TopHolders;

impl Tool for TopHolders {
    const NAME: &'static str = "top_holders";
    type Error = TopHoldersError;
    type Args = TopHoldersOperation;
    type Output = TopHoldersOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "top_holders".to_string(),
            description: "Check token concentration among top holders"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mint": {
                        "type": "string",
                        "description": "The mint address to check top holders for"
                    }
                },
                "required": ["mint"]
            }),
        }
    }

    async fn call(
        &self,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let mint = Pubkey::from_str(&args.mint)
            .map_err(|e| TopHoldersError::InvalidMint(e.to_string()))?;

        // Create a channel
        let (tx, mut rx) = mpsc::channel(1);

        // Spawn a task to handle the RPC calls
        tokio::spawn(async move {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let result = check_top_holders(&mint, &rpc_client, true).await;
            let _ = tx.send(result).await;
        });

        // Wait for the result
        let result = rx
            .recv()
            .await
            .ok_or_else(|| {
                TopHoldersError::CheckFailed("Channel closed".to_string())
            })?
            .map_err(|e| TopHoldersError::CheckFailed(e.to_string()))?;

        let (percentage, is_concentrated, details) = result;

        Ok(TopHoldersOutput {
            percentage,
            is_concentrated,
            details,
        })
    }
}

pub async fn make_agent(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = openai::Client::from_env();

    let agent = client
        .agent(openai::GPT_4O)
        .preamble("I can help you check token holder concentration.")
        .max_tokens(2048)
        .tool(TopHolders)
        .build();

    let holders_response = agent
        .prompt(
            "Check top holders for mint address GJAFwWjJ3vnTsrQVabjBVK2TYB1YtRCQXRDfDgUnpump, using the `top_holders` tool", 
        )
        .await?;

    let analysis = agent
        .prompt(&format!(
            "{}: {}",
            "is this a safe top 10 holders distribution?", holders_response
        ))
        .await?;

    println!("{}", analysis);

    Ok(())
}
