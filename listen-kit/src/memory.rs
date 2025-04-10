use listen_memory::memory_system::MemorySystem;
use listen_memory::query::generate_query;
use rig::message::Message;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn inject_memories(
    memory_system: Arc<MemorySystem>,
    prompt: String,
) -> anyhow::Result<String> {
    let query = generate_query(prompt.clone()).await?;
    let memories = memory_system.find_related_memories(query, 10).await?;
    let memory = memory_system
        .summarize_relevant_memories(memories, prompt.clone())
        .await?;
    let injected_prompt = format!(
        "<user-prompt>{}</user-prompt><relevant-memories>{}</relevant-memories>",
        prompt, memory
    );
    println!("injected_prompt: {}", injected_prompt);
    Ok(injected_prompt)
}

// TODO make this persistant and more elaborate
pub async fn synthesize_memories(
    memory_system: Arc<MemorySystem>,
    messages: Vec<Message>,
    dedup_set: Arc<RwLock<HashSet<String>>>,
) -> anyhow::Result<()> {
    for message in messages {
        match message {
            Message::Assistant { content, .. } => {
                let message_str = serde_json::to_string(&content)?;
                println!("message_str: {}", message_str);
                if !dedup_set.read().await.contains(&message_str) {
                    dedup_set.write().await.insert(message_str.clone());
                    memory_system.add_note(message_str).await?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_synthesize_memories() {
        let memory_system = MemorySystem::from_env().await.unwrap();
        let messages = vec![
            Message::user("what is the fartcoin catch phrase?"),
            Message::assistant("Fartcoin - hot air rises".to_string()),
        ];
        let dedup_set = Arc::new(RwLock::new(HashSet::new()));
        let res =
            synthesize_memories(Arc::new(memory_system), messages, dedup_set)
                .await;

        println!("res: {:?}", res);
        assert!(res.is_ok());
    }
}
