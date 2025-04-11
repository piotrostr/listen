pub fn must_get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        panic!("{} is not set", key);
    })
}
