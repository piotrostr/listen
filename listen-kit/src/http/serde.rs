use rig::message::AssistantContent;
use rig::message::Message;
use rig::message::ToolResult;
use rig::message::ToolResultContent;
use rig::message::UserContent;

use rig::OneOrMany;
use serde::de::Error;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct RawMessage {
    role: String,
    content: String,
}

/// Parse a single tool call from a JSON object
fn parse_tool_call<E>(
    id: String,
    name: String,
    params_str: &str,
) -> Result<Message, E>
where
    E: serde::de::Error,
{
    // Parse params string into a JSON value
    let arguments: Value = if params_str.is_empty() {
        json!({})
    } else {
        serde_json::from_str(params_str).map_err(|e| {
            E::custom(format!(
                "Failed to parse 'params' string for single tool call: {}",
                e
            ))
        })?
    };

    // Single tool calls must be from assistant role according to generation logic
    Ok(Message::Assistant {
        content: OneOrMany::one(AssistantContent::tool_call(
            id, name, arguments,
        )),
    })
}

/// Parse a single tool result from a JSON object
fn parse_tool_result<E>(
    id: String,
    result_content: String,
) -> Result<Message, E>
where
    E: serde::de::Error,
{
    // Single tool results must be from user role according to generation logic
    Ok(Message::User {
        content: OneOrMany::one(UserContent::ToolResult(ToolResult {
            id,
            content: OneOrMany::one(ToolResultContent::text(result_content)),
        })),
    })
}

/// Parse parallel tool calls from a JSON object
fn parse_parallel_tool_calls<E>(
    tool_calls_array: &[Value],
) -> Result<Message, E>
where
    E: serde::de::Error,
{
    let mut assistant_contents: Vec<AssistantContent> =
        Vec::with_capacity(tool_calls_array.len());

    for tool_call_value in tool_calls_array {
        if let Some(tool_call_obj) = tool_call_value.as_object() {
            let id = tool_call_obj
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    E::custom("Missing 'id' in parallel tool call")
                })?
                .to_string();

            let function_obj = tool_call_obj
                .get("function")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    E::custom(
                        "Missing 'function' object in parallel tool call",
                    )
                })?;

            let name = function_obj
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    E::custom("Missing 'name' in parallel tool call function")
                })?
                .to_string();

            let arguments =
                function_obj.get("arguments").cloned().ok_or_else(|| {
                    E::custom(
                        "Missing 'arguments' in parallel tool call function",
                    )
                })?;

            assistant_contents
                .push(AssistantContent::tool_call(id, name, arguments));
        } else {
            return Err(E::custom(
                "Invalid item in 'tool_calls' map: expected object",
            ));
        }
    }

    // Parallel tool calls must be from assistant role according to generation logic
    Ok(Message::Assistant {
        content: OneOrMany::many(assistant_contents).map_err(|e| {
            E::custom(format!(
                "Failed to create OneOrMany for assistant contents: {}",
                e
            ))
        })?,
    })
}

/// Parse parallel tool results from a JSON array
fn parse_parallel_tool_results<E>(
    tool_results_array: &[Value],
) -> Result<Message, E>
where
    E: serde::de::Error,
{
    let mut user_contents: Vec<UserContent> =
        Vec::with_capacity(tool_results_array.len());

    for tool_result_value in tool_results_array {
        if let Some(tool_result_obj) = tool_result_value.as_object() {
            let id = tool_result_obj
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    E::custom("Missing 'id' in parallel tool result")
                })?
                .to_string();

            let result_str = tool_result_obj.get("result")
                .and_then(Value::as_str)
                .ok_or_else(|| E::custom("Missing or invalid 'result' string in parallel tool result"))?
                .to_string();

            user_contents.push(UserContent::ToolResult(ToolResult {
                id,
                content: OneOrMany::one(ToolResultContent::text(result_str)),
            }));
        } else {
            return Err(E::custom(
                "Invalid item in 'tool_results' array: expected object",
            ));
        }
    }

    // Parallel tool results must be from user role according to generation logic
    Ok(Message::User {
        content: OneOrMany::many(user_contents).map_err(|e| {
            E::custom(format!(
                "Failed to create OneOrMany for user contents: {}",
                e
            ))
        })?,
    })
}

pub fn deserialize_messages<'de, D>(
    deserializer: D,
) -> Result<Vec<Message>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw_messages: Vec<RawMessage> = Vec::deserialize(deserializer)?;

    let res = raw_messages
        .into_iter()
        .map(|raw| -> Result<Message, D::Error> {
            // Attempt to parse content as JSON first
            dbg!(&raw);
            if let Ok(json_value) =
                serde_json::from_str::<Value>(&raw.content)
            {
                // --- Handle Parallel Tool Calls ---
                // Expected format: {"tool_calls": {"0": {"id": ..., "function": {"name":..., "arguments":...}}, ...}}
                if let Some(tool_calls_array) =
                    json_value.get("tool_calls").and_then(Value::as_array)
                {
                    return parse_parallel_tool_calls(tool_calls_array);
                }

                // --- Handle Parallel Tool Results ---
                // Expected format: {"tool_results": [{"index":..., "id": ..., "name": ..., "result": ...}, ...]}
                if let Some(tool_results_array) =
                    json_value.get("tool_results").and_then(Value::as_array)
                {
                    return parse_parallel_tool_results(tool_results_array);
                }

                // --- Handle Single Tool Result ---
                // Check this *after* parallel formats. Expected format: {"id": ..., "name": ..., "result": ...}
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("result").is_some()
                {
                    // This is a tool result - should always be a User message
                    let id_str = json_value["id"].as_str().unwrap();
                    let name_str = json_value["name"].as_str().unwrap();
                    let id = if id_str.is_empty() {
                        name_str.to_string()
                    } else {
                        id_str.to_string()
                    };
                    let result_content =
                        json_value["result"].as_str().unwrap().to_string();

                    return parse_tool_result(id, result_content);
                }

                // --- Handle Single Tool Call ---
                // Check this *after* parallel formats. Expected format: {"id": ..., "name": ..., "params": "{...}"}
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("params").is_some()
                {
                    // This is a tool call - should always be an Assistant message
                    let id = json_value["id"].as_str().unwrap().to_string();
                    let name =
                        json_value["name"].as_str().unwrap().to_string();
                    let params_str = json_value["params"].as_str().unwrap();

                    return parse_tool_call(id, name, params_str);
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
                _ => Err(D::Error::custom(format!(
                    "Invalid role: '{}'",
                    raw.role
                ))),
            }
        })
        .collect();

    dbg!(&res);

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_messages() {
        let raw_message = RawMessage {
            role: "assistant".to_string(),
            content: "{\"tool_results\":[{\"index\":0,\"id\":\"tool_0_get_sol_balance\",\"name\":\"get_sol_balance\",\"result\":\"64558916\"},{\"index\":1,\"id\":\"tool_1_get_spl_token_balance\",\"name\":\"get_spl_token_balance\",\"result\":\"[\\\"0\\\",6,\\\"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\\\"]\"},{\"index\":2,\"id\":\"tool_2_get_spl_token_balance\",\"name\":\"get_spl_token_balance\",\"result\":\"[\\\"0\\\",5,\\\"DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263\\\"]\"},{\"index\":3,\"id\":\"tool_3_get_spl_token_balance\",\"name\":\"get_spl_token_balance\",\"result\":\"[\\\"3815728090\\\",6,\\\"Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump\\\"]\"},{\"index\":4,\"id\":\"tool_4_get_spl_token_balance\",\"name\":\"get_spl_token_balance\",\"result\":\"ToolCallError: ToolCallError: Tool execution failed: \\\"Error {\\\\n    request: Some(\\\\n        GetTokenAccountBalance,\\\\n    ),\\\\n    kind: RpcError(\\\\n        RpcResponseError {\\\\n            code: -32602,\\\\n            message: \\\\\\\"Invalid param: could not find account\\\\\\\",\\\\n            data: Empty,\\\\n        },\\\\n    ),\\\\n}\\\"\"}]}".to_string(),
        };
        // nvm
    }
}
