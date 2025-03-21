use anyhow::Result;
use rig_tool_macro::tool;

#[tool(description = "
Use this tool as a scratchpad to think through complex problems without taking any external action.
It doesn't obtain new information or change any state, but provides a dedicated space
for reasoning.

Examples of when to use this tool:
- When you need to break down a complex request into steps
- When verifying compliance with specific policies or rules
- When analyzing outputs from previous tool calls
- When deciding between multiple possible approaches
")]
pub async fn think(thought: String) -> Result<String> {
    // This function doesn't perform any external actions
    // It simply returns confirmation that the thought was processed
    Ok("Thought processed".to_string())
}
