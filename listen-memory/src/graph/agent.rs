use super::tools::{
    add_memory_tool_graph, delete_memory_tool_graph, extract_entities_tool, noop_tool,
    relations_tool, update_memory_tool_graph,
};
use crate::util::must_get_env;
use anyhow::Result;
use serde_json::json;

pub fn geminifiy_tool_definition(tool: &serde_json::Value) -> serde_json::Value {
    let function = tool["function"].as_object().unwrap();
    json!({
        "name": function["name"],
        "description": function["description"],
        "parameters": function["parameters"],
    })
}

pub async fn get_tool_calls(
    input: String,
    system_prompt: String,
    tools: Vec<serde_json::Value>,
) -> Result<serde_json::Value> {
    let api_key = must_get_env("GEMINI_API_KEY");
    let client = reqwest::Client::new();
    let response = client
        .post(&format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            api_key
        ))
        .json(&json!({
            "systemInstruction": {
                "parts": [
                    {
                        "text": system_prompt,
                    }
                ]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {
                            "text": input,
                        }
                    ],
                }
            ],
            "tools": [
                {
                    "functionDeclarations": tools.iter().map(geminifiy_tool_definition).collect::<Vec<_>>(),
                }
            ]
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    // println!("{}", serde_json::to_string_pretty(&response_json).unwrap());

    Ok(response_json)
}

pub fn extract_tool_calls(response: &serde_json::Value) -> Result<Vec<serde_json::Value>> {
    let parts = response["candidates"][0]["content"]["parts"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No tool calls found"))?;
    let tool_calls = parts
        .iter()
        .map(|part| part["functionCall"].clone())
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string_pretty(&tool_calls).unwrap());

    Ok(tool_calls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_with_tools() {
        let input = "Paris is the capital of France, France is next to Germany";
        let tools = vec![add_memory_tool_graph()];
        let result = get_tool_calls(
            input.to_string(),
            "You are a helpful assistant that can add memory to a graph".to_string(),
            tools,
        )
        .await
        .unwrap();
        let tool_calls = extract_tool_calls(&result).unwrap();
        println!("{:?}", tool_calls);
    }
}
