use rig::message::AssistantContent;
use rig::message::Message;
use rig::message::ToolResult;
use rig::message::ToolResultContent;
use rig::message::UserContent;

use rig::OneOrMany;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

pub fn deserialize_messages<'de, D>(
    deserializer: D,
) -> Result<Vec<Message>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize, Serialize, Debug)]
    struct RawMessage {
        role: String,
        content: String,
    }

    let raw_messages: Vec<RawMessage> = Vec::deserialize(deserializer)?;

    raw_messages
        .into_iter()
        .map(|raw| -> Result<Message, D::Error> {
            // Attempt to parse content as JSON first
            if let Ok(json_value) = serde_json::from_str::<Value>(&raw.content) {
                // --- Handle Parallel Tool Calls ---
                // Expected format: {"tool_calls": {"0": {"id": ..., "function": {"name":..., "arguments":...}}, ...}}
                if let Some(tool_calls_map) = json_value.get("tool_calls").and_then(Value::as_object) {
                    let mut assistant_contents: Vec<AssistantContent> = Vec::with_capacity(tool_calls_map.len());
                    for tool_call_value in tool_calls_map.values() {
                        if let Some(tool_call_obj) = tool_call_value.as_object() {
                            let id = tool_call_obj.get("id").and_then(Value::as_str).ok_or_else(|| serde::de::Error::custom("Missing 'id' in parallel tool call"))?.to_string();
                            let function_obj = tool_call_obj.get("function").and_then(Value::as_object).ok_or_else(|| serde::de::Error::custom("Missing 'function' object in parallel tool call"))?;
                            let name = function_obj.get("name").and_then(Value::as_str).ok_or_else(|| serde::de::Error::custom("Missing 'name' in parallel tool call function"))?.to_string();
                            let arguments = function_obj.get("arguments").cloned().ok_or_else(|| serde::de::Error::custom("Missing 'arguments' in parallel tool call function"))?;

                            assistant_contents.push(AssistantContent::tool_call(id, name, arguments));
                        } else {
                            return Err(serde::de::Error::custom("Invalid item in 'tool_calls' map: expected object"));
                        }
                    }
                    // Parallel tool calls *must* be from assistant role according to generation logic
                    return Ok(Message::Assistant {
                        content: OneOrMany::many(assistant_contents).map_err(|e| serde::de::Error::custom(format!("Failed to create OneOrMany for assistant contents: {}", e)))?,
                    });
                }

                // --- Handle Parallel Tool Results ---
                // Expected format: {"tool_results": [{"index":..., "id": ..., "name": ..., "result": ...}, ...]}
                if let Some(tool_results_array) = json_value.get("tool_results").and_then(Value::as_array) {
                    let mut user_contents: Vec<UserContent> = Vec::with_capacity(tool_results_array.len());
                    for tool_result_value in tool_results_array {
                        if let Some(tool_result_obj) = tool_result_value.as_object() {
                            let id = tool_result_obj.get("id").and_then(Value::as_str).ok_or_else(|| serde::de::Error::custom("Missing 'id' in parallel tool result"))?.to_string();
                            let result_str = tool_result_obj.get("result").and_then(Value::as_str).ok_or_else(|| serde::de::Error::custom("Missing or invalid 'result' string in parallel tool result"))?.to_string();

                            user_contents.push(UserContent::ToolResult(ToolResult {
                                id,
                                content: OneOrMany::one(ToolResultContent::text(result_str)),
                            }));
                        } else {
                            return Err(serde::de::Error::custom("Invalid item in 'tool_results' array: expected object"));
                        }
                    }
                    // Parallel tool results *must* be from user role according to generation logic
                    return Ok(Message::User {
                        content: OneOrMany::many(user_contents).map_err(|e| serde::de::Error::custom(format!("Failed to create OneOrMany for user contents: {}", e)))?,
                    });
                }

                // --- Handle Single Tool Result (Existing Logic) ---
                // Check this *after* parallel formats. Expected format: {"id": ..., "name": ..., "result": ...}
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("result").is_some()
                {
                    // This is a tool result - should always be a User message
                    let id_str = json_value["id"].as_str().unwrap();
                    let name_str = json_value["name"].as_str().unwrap();
                    let id = if id_str.is_empty() { name_str.to_string() } else { id_str.to_string() };
                    let result_content = json_value["result"].as_str().unwrap().to_string();

                    // Single tool results *must* be from user role according to generation logic
                    return Ok(Message::User {
                        content: OneOrMany::one(UserContent::ToolResult(
                            ToolResult {
                                id,
                                content: OneOrMany::one(
                                    ToolResultContent::text(result_content),
                                ),
                            },
                        )),
                    });
                }

                // --- Handle Single Tool Call (Existing Logic) ---
                // Check this *after* parallel formats. Expected format: {"id": ..., "name": ..., "params": "{...}"}
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("params").is_some()
                {
                    // This is a tool call - should always be an Assistant message
                    let id = json_value["id"].as_str().unwrap().to_string();
                    let name = json_value["name"].as_str().unwrap().to_string();
                    let params_str = json_value["params"].as_str().unwrap();

                    // Parse params string into a JSON value
                    let arguments: Value = if params_str.is_empty() {
                        json!({})
                    } else {
                        serde_json::from_str(params_str)
                             .map_err(|e| serde::de::Error::custom(format!("Failed to parse 'params' string for single tool call: {}", e)))?
                    };

                    // Single tool calls *must* be from assistant role according to generation logic
                    return Ok(Message::Assistant {
                        content: OneOrMany::one(AssistantContent::tool_call(
                            id, name, arguments,
                        )),
                    });
                }
            }

            // --- Fallback to Regular Text Messages ---
            // If content wasn't JSON or didn't match tool formats, treat as text based on role.
            match raw.role.as_str() {
                "user" => Ok(Message::User {
                    content: OneOrMany::one(UserContent::Text(
                        raw.content.into(),
                    )),
                }),
                "assistant" => Ok(Message::Assistant {
                    content: OneOrMany::one(AssistantContent::text(
                        raw.content,
                    )),
                }),
                _ => Err(serde::de::Error::custom(format!("Invalid role: '{}'", raw.role))),
            }
        })
        .collect()
}
