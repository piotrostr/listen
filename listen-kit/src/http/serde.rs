use rig::message::AssistantContent;
use rig::message::Message;
use rig::message::ToolResult;
use rig::message::ToolResultContent;
use rig::message::UserContent;

use rig::OneOrMany;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

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
        .map(|raw| {
            // First check if this is a tool result (regardless of role)
            if let Ok(json_value) =
                serde_json::from_str::<serde_json::Value>(&raw.content)
            {
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("result").is_some()
                {
                    // This is a tool result - should always be a User message
                    let id = json_value["id"].as_str().unwrap().to_string();
                    let result_content =
                        json_value["result"].as_str().unwrap().to_string();

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

                // Check if this is a tool call (regardless of role)
                if json_value.is_object()
                    && json_value.get("id").is_some()
                    && json_value.get("name").is_some()
                    && json_value.get("params").is_some()
                {
                    // This is a tool call - should always be an Assistant message
                    let id = json_value["id"].as_str().unwrap().to_string();
                    let name =
                        json_value["name"].as_str().unwrap().to_string();
                    let params = json_value["params"].as_str().unwrap();

                    // Parse params into a JSON object
                    let arguments: serde_json::Value = if params.is_empty() {
                        json!({})
                    } else {
                        serde_json::from_str(params)
                            .unwrap_or_else(|_| json!({}))
                    };

                    return Ok(Message::Assistant {
                        content: OneOrMany::one(AssistantContent::tool_call(
                            id, name, arguments,
                        )),
                    });
                }
            }

            // For regular messages, use the role as specified
            match raw.role.as_str() {
                "user" => Ok(Message::User {
                    content: OneOrMany::one(UserContent::Text(
                        raw.content.into(),
                    )),
                }),
                "assistant" => Ok(Message::Assistant {
                    content: OneOrMany::one(raw.content.into()),
                }),
                _ => Err(serde::de::Error::custom("Invalid role")),
            }
        })
        .collect()
}
