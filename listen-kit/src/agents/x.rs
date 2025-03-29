use crate::common::{gemini_agent_builder, GeminiAgent};
use crate::data::twitter_tools::{
    FetchXPost, ResearchXProfile, SearchTweets,
};

pub fn create_x_agent() -> GeminiAgent {
    gemini_agent_builder()
        .preamble("You are an expert at researching x profiles, posts, and tweets. Use your tools to get as much output for the given prompt as possible. If a tool output yields more outputs, continue to explore")
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .build()
}
