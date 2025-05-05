use anyhow::Result;
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
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let (engine, _) = Engine::from_env().await?;

    let id = Uuid::from_str("94773d48-0ed9-450f-a8a5-31d5b3105ae1").unwrap();
    let step_id = Uuid::from_str("c249e16a-8082-474c-b7c7-09a13ec01169").unwrap();
    let user_id = "did:privy:cm6cxky3i00ondmuatkemmffm".to_string();

    let mut steps = HashMap::new();
    steps.insert(
        step_id,
        PipelineStep {
            id: Uuid::from_str("d08234f7-e960-4bac-be54-f27e3afce6ad").unwrap(),
            action: Action::Order(SwapOrder {
                input_token: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
                output_token: "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf".to_string(),
                amount: "2467501".to_string(),
                from_chain_caip2: "eip155:8453".to_string(),
                to_chain_caip2: "eip155:8453".to_string(),
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
