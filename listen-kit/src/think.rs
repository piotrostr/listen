use anyhow::Result;
use rig_tool_macro::tool;

#[tool(description = "
Use the tool to think about something, treat it like a scratchpad. It will not
obtain new information or change any state, but just append the thought to the
log. Use it when complex reasoning or some cache memory is needed.
")]
pub async fn think(#[allow(unused)] thought: String) -> Result<String> {
    // This function doesn't perform any external actions
    // It simply returns confirmation that the thought was processed
    Ok("Thought processed".to_string())
}
