use regex::Regex;

pub fn must_get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        panic!("{} is not set", key);
    })
}

pub fn extract_from_code_blocks_if_any(content: &str) -> String {
    // Match everything between triple backticks, non-greedy
    let re = Regex::new(r"```(?:\w+)?\s*([\s\S]*?)\s*```").unwrap();
    let caps = re.captures(content);
    let result = caps.map_or_else(|| content.trim().to_string(), |c| c[1].trim().to_string());
    result
}
