use crate::reasoning_loop::StreamResponse;

pub fn refresh_accumulated_message(
    message_acc: &mut String,
    output_responses: &mut Vec<StreamResponse>,
) {
    if !message_acc.is_empty() {
        output_responses.push(StreamResponse::Message(message_acc.clone()));
        message_acc.clear();
    }
}

/// helper to aggregate streamed message chunks into one and respect breaks on tool call/output
pub fn join_responses(
    input_responses: Vec<StreamResponse>,
) -> Vec<StreamResponse> {
    let mut output_responses = Vec::new();
    let mut message_acc = String::new();

    for response in input_responses {
        match response {
            StreamResponse::ParToolCall { tool_calls } => {
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses
                    .push(StreamResponse::ParToolCall { tool_calls });
            }
            StreamResponse::ParToolResult { tool_results } => {
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses
                    .push(StreamResponse::ParToolResult { tool_results });
            }
            StreamResponse::Message(message) => {
                message_acc.push_str(&message);
            }
            StreamResponse::NestedAgentOutput {
                agent_type,
                content,
            } => {
                // Pass through nested agent outputs unchanged
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses.push(StreamResponse::NestedAgentOutput {
                    agent_type,
                    content,
                });
            }
            StreamResponse::ToolCall { id, name, params } => {
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses.push(StreamResponse::ToolCall {
                    id,
                    name,
                    params,
                });
            }
            StreamResponse::ToolResult { id, name, result } => {
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses.push(StreamResponse::ToolResult {
                    id,
                    name,
                    result,
                });
            }
            StreamResponse::Error(error) => {
                refresh_accumulated_message(
                    &mut message_acc,
                    &mut output_responses,
                );
                output_responses.push(StreamResponse::Error(error));
            }
        }
    }

    // Add any remaining accumulated message
    if !message_acc.is_empty() {
        output_responses.push(StreamResponse::Message(message_acc));
    }

    output_responses
}
