use std::{collections::HashMap, str::FromStr};

use listen_engine::{
    engine::{
        order::SwapOrder,
        pipeline::{Action, Condition, ConditionType, Pipeline, PipelineStep, Status},
    },
    Engine,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    let (engine, _) = Engine::from_env().await?;
    let id = Uuid::from_str("71bcf378-41a0-4860-b29d-a99947786837").unwrap();
    let user_id = "did:privy:cm6cxky3i00ondmuatkemmffm".to_string();
    let step_id = Uuid::from_str("af8c7a8c-50cd-4273-9c6a-034d60884b5b").unwrap();
    let mut steps = HashMap::new();

    let solana_balance = reqwest::Client::new()
        .post(std::env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set"))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [
                "6fp9frQ16W3kTRGiBVvpMS2NzoixE4Y1MWqYrW9SvTAj",
                {"commitment": "confirmed"}
            ],
            "id": 1
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    tracing::info!("Solana balance: {:#?}", solana_balance);

    steps.insert(
        step_id,
        PipelineStep {
            id: step_id,
            action: Action::Order(SwapOrder {
                input_token: "So11111111111111111111111111111111111111112".to_string(),
                output_token: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(),
                amount: "20000000".to_string(), // 0.02 SOL
                from_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                to_chain_caip2: "eip155:42161".to_string(),
            }),
            conditions: vec![Condition {
                condition_type: ConditionType::Now {
                    asset: "".to_string(),
                },
                triggered: false,
                last_evaluated: None,
            }],
            next_steps: vec![],
            status: Status::Pending,
            transaction_hash: None,
            error: None,
        },
    );
    let mut pipeline = Pipeline {
        id,
        user_id,
        wallet_address: Some("0xCCC48877a33a2C14e40c82da843Cf4c607ABF770".to_string()),
        pubkey: Some("6fp9frQ16W3kTRGiBVvpMS2NzoixE4Y1MWqYrW9SvTAj".to_string()),
        current_steps: vec![step_id],
        steps,
        status: Status::Pending,
        created_at: chrono::Utc::now(),
    };
    engine.evaluate_pipeline(&mut pipeline).await.unwrap();
    Ok(())
}
